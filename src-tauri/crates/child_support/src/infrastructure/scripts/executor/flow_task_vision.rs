impl ScriptExecutor {
    async fn execute_task_control_step(
        &mut self,
        task_control: &TaskControl,
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence("taskControl.setState", "TaskControl 状态写入")
            .await?
        {
            return Ok(timeout_flow);
        }
        match task_control {
            TaskControl::SetState {
                target,
                targets,
                status,
            } => {
                if targets.is_empty() {
                    self.set_state_value(target, status).await?;
                } else {
                    for target in targets {
                        self.set_state_value(target, status).await?;
                    }
                }
                Ok(ControlFlow::Next)
            }
        }
    }

    async fn execute_vision_step(&mut self, vision: &VisionNode) -> ExecuteResult<ControlFlow> {
        match vision {
            VisionNode::VisionSearch {
                rule,
                out_var,
                then_steps,
            } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "vision.search",
                        format!("VisionSearch 搜索并写入 {}", out_var),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                let (hits, matched) = {
                    let ctx = self.runtime_ctx.read().await;
                    if let Some(snapshot) = ctx.observation.last_snapshot.as_ref() {
                        let searcher = OcrSearcher::new(std::slice::from_ref(rule));
                        let hits = searcher.search(snapshot);
                        let matched = rule.evaluate(&hits, &snapshot.det_items);
                        (hits, matched)
                    } else {
                        (Vec::new(), false)
                    }
                };

                {
                    let mut ctx = self.runtime_ctx.write().await;
                    ctx.observation.last_hits = hits.clone();
                }

                self.set_runtime_var(out_var, Self::search_hits_to_dynamic(&hits))
                    .await?;

                if matched && !then_steps.is_empty() {
                    return self.execute(then_steps).await;
                }

                Ok(ControlFlow::Next)
            }
        }
    }
}
