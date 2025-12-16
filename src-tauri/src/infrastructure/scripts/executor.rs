use rhai::{Engine, Scope, Dynamic};
use crate::domain::scripts::script_decision::Step;
use crate::infrastructure::scripts::script_error::{ExecuteResult, ScriptError};

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
    engine: Engine,
    // Scope is generic, but usually we can use a fresh scope or keep one.
    // For a stateful script, we keep one scope.
    // Note: Rhai Scope is not thread-safe and not Send/Sync by default usually?
    // Actually Scope works fine in a single thread execution loop.
    scope: Scope<'static>, 
}

impl ScriptExecutor {
    pub fn new() -> Self {
        let engine = Engine::new();
        Self {
            engine,
            scope: Scope::new(),
        }
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

    fn execute_step<'a>(&'a mut self, step: &'a Step) -> Pin<Box<dyn Future<Output = ExecuteResult<ControlFlow>> + 'a>> {
        Box::pin(async move {
            match step {
                Step::Sequence { steps, reverse } => {
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
                Step::Continue => return Ok(ControlFlow::Continue),
                Step::Break => return Ok(ControlFlow::Break),
                Step::If { cond, then_steps, else_steps } => {
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
                Step::While { cond, steps, max_loop } => {
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
                Step::SetVar { name, value_expr } => {
                     let val: Dynamic = self.engine.eval_expression_with_scope(&mut self.scope, value_expr)
                        .map_err(|e| ScriptError::ExecuteErr { step_type: "setVar".to_string(), e: e.to_string() })?;
                     self.scope.set_value(name, val);
                }
                Step::GetVar { name } => {
                    // Usually for debugging or returning? 
                    // In this design, GetVar might not do much unless logged.
                    if let Some(val) = self.scope.get_value::<Dynamic>(name) {
                        tracing::info!("Var {}: {:?}", name, val);
                    }
                }
                Step::WaitMs { ms } => {
                    tokio::time::sleep(Duration::from_millis(*ms)).await;
                }
                Step::WaitUntil { cond, timeout_ms } => {
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
                Step::TakeScreenshot { output_var } => {
                    // TODO: Call device.screencap()
                    // Store path or handle in scope
                    self.scope.set_value(output_var, "placeholder_image_path.png".to_string());
                }
                Step::Ocr { image_var, output_var } => {
                     // TODO: Call OCR engine
                     self.scope.set_value(output_var, "detected text".to_string());
                }
                Step::FindObject { image_var, query, output_var } => {
                     // TODO: Call vision 
                     self.scope.set_value(output_var,  "100,200".to_string());
                }
                Step::Click { target_var, .. } => {
                     // TODO: Click
                     if let Some(var_name) = target_var {
                         // get coords from var
                     }
                }
                 _ => {}
            }
            Ok(ControlFlow::Next)
        })
    }
}
