use rhai::{Engine, Scope, Dynamic};
use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::scripts::script_error::{ExecuteResult, ScriptError};
use crate::infrastructure::context::runtime_context::SharedRuntimeContext;
use crate::infrastructure::core::StepId;
use tokio::time::Duration;
use std::pin::Pin;
use std::future::Future;

#[derive(Debug)]
pub enum ControlFlow {
    Continue,
    Break,
    Next,
    Return,
}

pub struct ScriptExecutor {
    pub engine: Engine,
    pub scope: Scope<'static>,
    pub runtime_ctx: SharedRuntimeContext,
    pub node_indices: crate::infrastructure::core::HashMap<StepId, usize>,
}

impl ScriptExecutor {
    pub fn new(runtime_ctx: SharedRuntimeContext) -> Self {
        let engine = Engine::new();
        
        Self {
            engine,
            scope: Scope::new(),
            runtime_ctx,
            node_indices: crate::infrastructure::core::HashMap::new(),
        }
    }

    pub fn reset_node_indices(&mut self) {
        self.node_indices.clear();
    }

    pub fn get_node_index(&self, id: &StepId) -> usize {
        self.node_indices.get(id).cloned().unwrap_or(0)
    }

    pub fn set_node_index(&mut self, id: &StepId, val: usize) {
        self.node_indices.insert(id.clone(), val);
    }

    pub fn inc_node_index(&mut self, id: &StepId, amount: usize) {
        let current = self.get_node_index(id);
        self.set_node_index(id, current + amount);
    }

    pub fn reset_scope(&mut self) {
        self.scope.clear();
    }

    pub async fn execute(&mut self, steps: &[Step]) -> ExecuteResult<ControlFlow> {
        for step in steps {
            match self.execute_step(step).await? {
                ControlFlow::Next => continue,
                ControlFlow::Continue => return Ok(ControlFlow::Continue),
                ControlFlow::Break => return Ok(ControlFlow::Break),
                ControlFlow::Return => return Ok(ControlFlow::Return),
            }
        }
        Ok(ControlFlow::Next)
    }

    fn execute_step<'a>(&'a mut self, step: &'a crate::domain::scripts::script_decision::Step) -> Pin<Box<dyn Future<Output = ExecuteResult<ControlFlow>> + 'a>> {
        Box::pin(async move {
            use crate::domain::scripts::script_decision::StepKind;
            
            // 每次执行步骤前，如果步骤有 ID，可以将当前索引注入到 Rhai Scope
            if let Some(id) = &step.id {
                let idx = self.get_node_index(id);
                self.scope.set_value(format!("idx_{}", id), idx as i64);
            }

            match &step.kind {
                StepKind::Sequence { steps, reverse } => {
                    let iter: Box<dyn Iterator<Item = _>> = if *reverse {
                        Box::new(steps.iter().rev())
                    } else {
                        Box::new(steps.iter())
                    };
                    
                    for s in iter {
                        let flow = self.execute_step(s).await?;
                        match flow {
                            ControlFlow::Next => continue,
                            _ => return Ok(flow), // Propagate Break/Continue/Return
                        }
                    }
                }
                StepKind::Continue => return Ok(ControlFlow::Continue),
                StepKind::Break => return Ok(ControlFlow::Break),
                StepKind::If { cond, then_steps, else_steps } => {
                    let val: bool = self.engine.eval_expression_with_scope(&mut self.scope, cond)
                        .map_err(|e| ScriptError::ExecuteErr { step_type: "if".to_string(), e: e.to_string() })?;
                    
                    if val {
                        // then_steps is Vec<Step>
                        for s in then_steps {
                            let flow = self.execute_step(s).await?;
                            if matches!(flow, ControlFlow::Break | ControlFlow::Continue | ControlFlow::Return) {
                                return Ok(flow);
                            }
                        }
                    } else if let Some(else_block) = else_steps {
                        // else_steps is Box<Step>, likely a Sequence or single step
                        let flow = self.execute_step(else_block).await?;
                         if matches!(flow, ControlFlow::Break | ControlFlow::Continue | ControlFlow::Return) {
                            return Ok(flow);
                        }
                    }
                }
                StepKind::While { cond, steps, max_loop } => {
                    let mut count = 0;
                    loop {
                        // Check max_loop
                        if let Some(max) = max_loop {
                            if count >= *max {
                                break;
                            }
                        }
                        
                        // Check condition
                        let val: bool = self.engine.eval_expression_with_scope(&mut self.scope, cond)
                             .map_err(|e| ScriptError::ExecuteErr { step_type: "while".to_string(), e: e.to_string() })?;
                        
                        if !val {
                            break;
                        }

                        // Execute body
                        let mut broken = false;
                        for s in steps {
                            let flow = self.execute_step(s).await?;
                            match flow {
                                ControlFlow::Break => {
                                    broken = true;
                                    break;
                                },
                                ControlFlow::Continue => break, // Stop current iteration, go to next
                                ControlFlow::Return => return Ok(ControlFlow::Return),
                                ControlFlow::Next => {},
                            }
                        }
                        if broken {
                            break;
                        }
                        count += 1;
                    }
                }
                StepKind::SetVar { name, value_expr } => {
                     let val: Dynamic = self.engine.eval_expression_with_scope(&mut self.scope, value_expr)
                        .map_err(|e| ScriptError::ExecuteErr { step_type: "setVar".to_string(), e: e.to_string() })?;
                     self.scope.set_value(name, val);
                }
                StepKind::GetVar { name } => {
                    // Usually for debugging or returning? 
                    // In this design, GetVar might not do much unless logged.
                    if let Some(val) = self.scope.get_value::<Dynamic>(name) {
                        tracing::info!("Var {}: {:?}", name, val);
                    }
                }
                StepKind::WaitMs { ms } => {
                    tokio::time::sleep(Duration::from_millis(*ms)).await;
                }
                StepKind::WaitUntil { cond, timeout_ms } => {
                     let start = tokio::time::Instant::now();
                     let timeout = Duration::from_millis(*timeout_ms);
                     
                     loop {
                         let val: bool = self.engine.eval_expression_with_scope(&mut self.scope, cond)
                            .unwrap_or(false);
                         if val {
                             break;
                         }
                         if start.elapsed() > timeout {
                             break; // Timeout, maybe return error or bool?
                         }
                         tokio::time::sleep(Duration::from_millis(100)).await;
                     }
                }
                
                // Vision/Device ops - Placeholder for now as we need Device Traits
                StepKind::TakeScreenshot { output_var } => {
                    // TODO: Call device.screencap()
                    // Store path or handle in scope
                    self.scope.set_value(output_var, "placeholder_image_path.png".to_string());
                    let mut ctx = self.runtime_ctx.write().await;
                    ctx.last_snapshot = None; // 截图后旧快照失效
                }
                StepKind::Ocr { image_var: _, output_var } => {
                     // TODO: Call OCR engine
                     self.scope.set_value(output_var, "detected text".to_string());
                }
                StepKind::FindObject { image_var: _, query: _, output_var } => {
                     // TODO: Call vision 
                     self.scope.set_value(output_var,  "100,200".to_string());
                }
                StepKind::ClickAction(_click) => {
                     // TODO: Click
                    /*match click {
                        Click::Label{
                            label,
                            label_idx,
                            ..
                        } => {
                            // 获取坐标
                            if let Some(coords) = self.scope.get_value::<String>(target_var) {
                                let coords: Vec<&str> = coords.split(',').collect();
                                let x: i32 = coords[0].parse().unwrap();
                                let y: i32 = coords[1].parse().unwrap();
                                // TODO: Click
                                self.adb_executor
                            }
                        },
                        _ => {}
                    }*/
                }
                StepKind::VisionSearch { rule, output_var } => {
                    let mut ctx = self.runtime_ctx.write().await;
                    
                    // 1. 获取当前快照 (如果过期则重建)
                    if ctx.last_snapshot.is_none() {
                        // TODO: 从 adb_executor 获取截图并跑 OCR/YOLO
                        // 这里需要整合 adb_executor.capture()
                    }

                    let hits = if let Some(snapshot) = &ctx.last_snapshot {
                        let mut rules = Vec::new();
                        rules.push(rule.clone());
                        let searcher = crate::domain::vision::ocr_search::OcrSearcher::new(&rules);
                        Some(searcher.search(snapshot))
                    } else {
                        None
                    };

                    if let Some(hits) = hits {
                        // 将命中结果存入缓存
                        let success = rule.evaluate(&hits);
                        ctx.last_hits = hits;
                        // 将命中结果存入变量
                        self.scope.set_value(output_var, success);
                    }
                }
                // 索引管理
                StepKind::IncIndex { id, amount } => {
                    self.inc_node_index(id, amount.unwrap_or(1));
                }
                StepKind::ResetIndex { id } => {
                    if let Some(sid) = id {
                        self.node_indices.remove(sid);
                    } else {
                        self.reset_node_indices();
                    }
                }
                // 滑动重置索引
                StepKind::SwipeDet { .. } | StepKind::SwipeTxt { .. } | StepKind::SwipePoint { .. } | StepKind::SwipePercent { .. } => {
                    self.reset_node_indices();
                    // TODO: 实际滑动逻辑
                }
                StepKind::SetState { .. } => {
                     // TODO: Set state in global context
                }
                StepKind::GetState { .. } => {
                     // TODO: Get state from global context
                }
                StepKind::StopPolicy => {
                     return Ok(ControlFlow::Return);
                }
                StepKind::FinishTask { .. } => {
                     return Ok(ControlFlow::Return);
                }
                StepKind::FilterHits { .. } => {
                     // TODO: Implementation with Rhai
                }
                _ => {}
            }
            
            // 自动迭代逻辑
            /*if step.iterate {
                if let Some(id) = &step.id {
                    self.inc_node_index(id, 1);
                }
            }*/

            Ok(ControlFlow::Next)
        })
    }
}
