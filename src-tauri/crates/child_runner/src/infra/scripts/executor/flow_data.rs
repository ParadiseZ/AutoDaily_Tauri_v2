impl ScriptExecutor {
    async fn execute_data_handling_step(
        &mut self,
        data: &DataHanding,
    ) -> ExecuteResult<ControlFlow> {
        match data {
            DataHanding::SetVar {
                name,
                val,
                json_val,
                expr,
            } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence("data.setVar", format!("SetVar 写入变量 {}", name))
                    .await?
                {
                    return Ok(timeout_flow);
                }
                let value = if let Some(json_val) = json_val {
                    Self::json_value_to_dynamic("data.setVar", "JSON值", json_val)?
                } else if let Some(expr) = expr.as_ref().filter(|value| !value.trim().is_empty()) {
                    self.eval_dynamic(expr, "data.setVar")?
                } else if let Some(val) = val {
                    Self::var_value_to_dynamic(val)
                } else {
                    Dynamic::UNIT
                };
                self.set_runtime_var(name, value).await?;
                Ok(ControlFlow::Next)
            }
            DataHanding::ClearVars { names } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "data.clearVars",
                        format!("ClearVars 清空 {} 个变量", names.len()),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                for name in names.iter().filter(|name| !name.trim().is_empty()) {
                    self.clear_runtime_var_value(name).await?;
                }
                Ok(ControlFlow::Next)
            }
            DataHanding::GetVar { name, default_val } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence("data.getVar", format!("GetVar 读取变量 {}", name))
                    .await?
                {
                    return Ok(timeout_flow);
                }
                if self.read_runtime_var(name).await.is_none() {
                    if let Some(default_val) = default_val {
                        self.set_runtime_var(name, Self::var_value_to_dynamic(default_val))
                            .await?;
                    }
                }
                Ok(ControlFlow::Next)
            }
            DataHanding::Print {
                source,
                value,
                level,
            } => {
                let output = match source {
                    PrintSource::Text => value.clone(),
                    PrintSource::Variable => self
                        .read_runtime_var(value)
                        .await
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| format!("<变量 {} 未定义>", value)),
                };
                let message = format!("[ script ] {}", output);
                #[cfg(feature = "testkit")]
                if let Some(test_hooks) = self.test_hooks.as_ref() {
                    test_hooks.record_print(level, output.clone()).await;
                }
                match level {
                    LogLevel::Debug => Log::debug(&message),
                    LogLevel::Info => Log::info(&message),
                    LogLevel::Warn => Log::warn(&message),
                    LogLevel::Error => Log::error(&message),
                    LogLevel::Off => {}
                }
                Ok(ControlFlow::Next)
            }
            DataHanding::Filter {
                input_var,
                out_name,
                mode,
                logic_expr,
                region_top_left,
                region_bottom_right,
                then_steps,
            } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence(
                        "data.filter",
                        format!("Filter 准备处理输入变量 {}", input_var),
                    )
                    .await?
                {
                    return Ok(timeout_flow);
                }
                let Some(input) = self.read_runtime_var(input_var).await else {
                    self.set_runtime_var(out_name, Dynamic::from(Array::new()))
                        .await?;
                    return Ok(ControlFlow::Next);
                };

                let Some(items) = input.clone().try_cast::<Array>() else {
                    return Err(Self::execute_error(
                        "data.filter",
                        format!("输入变量[{}]不是数组，无法执行过滤", input_var),
                    ));
                };

                let mut output = Array::new();
                let region = self
                    .resolve_region_rect(region_top_left, region_bottom_right)
                    .await?;
                for (index, item) in items.into_iter().enumerate() {
                    if let Some(timeout_flow) = self
                        .record_progress_evidence(
                            "data.filter.item",
                            format!("Filter 处理条目 {} -> {}", input_var, index),
                        )
                        .await?
                    {
                        return Ok(timeout_flow);
                    }
                    self.scope.set_value(FILTER_ITEM_VAR, item.clone());
                    self.scope.set_value(ITEM_VAR, item.clone());
                    self.scope.set_value(FILTER_INDEX_VAR, index as i64);
                    self.scope.set_value(ITEM_INDEX_VAR, index as i64);

                    let matched = if logic_expr.trim().is_empty() {
                        true
                    } else {
                        self.eval_bool(logic_expr, "data.filter.logicExpr")?
                    };

                    if !matched {
                        continue;
                    }

                    if let Some(region) = region.as_ref() {
                        if !Self::dynamic_item_in_region(&item, region)? {
                            continue;
                        }
                    }

                    if !then_steps.is_empty() {
                        match self.execute(then_steps).await? {
                            ControlFlow::Next => {}
                            ControlFlow::Continue => continue,
                            ControlFlow::Break => break,
                            ControlFlow::Link(target) => return Ok(ControlFlow::Link(target)),
                            ControlFlow::Return => return Ok(ControlFlow::Return),
                            ControlFlow::StopScript => return Ok(ControlFlow::StopScript),
                        }
                    }

                    match mode {
                        FilterMode::Filter => output.push(item),
                        FilterMode::Map => {
                            let current = self
                                .scope
                                .get_value::<Dynamic>(ITEM_VAR)
                                .unwrap_or_else(|| Dynamic::UNIT);
                            output.push(current);
                        }
                    }
                }

                self.set_runtime_var(out_name, Dynamic::from(output))
                    .await?;
                Ok(ControlFlow::Next)
            }
            DataHanding::ColorCompare {
                input_var,
                out_var,
                target_text,
                is_font,
                target_color,
                method,
                region_top_left,
                region_bottom_right,
                then_steps,
            } => {
                let region = self
                    .resolve_region_rect(region_top_left, region_bottom_right)
                    .await?;
                self.execute_color_compare_step(
                    input_var,
                    out_var,
                    target_text.as_deref(),
                    *is_font,
                    target_color,
                    method,
                    region.as_ref(),
                    then_steps,
                )
                .await
            }
            DataHanding::RelativeFilter {
                input_var,
                out_var,
                anchor_type,
                anchor_text,
                anchor_idx,
                direction,
                target_kind,
                max_offset_x,
                max_offset_y,
                target_index,
                then_steps,
            } => {
                self.execute_relative_filter_step(
                    input_var,
                    out_var,
                    anchor_type,
                    anchor_text,
                    *anchor_idx,
                    direction,
                    target_kind,
                    *max_offset_x,
                    *max_offset_y,
                    *target_index,
                    then_steps,
                )
                .await
            }
            DataHanding::Rhai { code, out_var } => {
                if let Some(timeout_flow) = self
                    .record_progress_evidence("data.rhai", "执行 Rhai 代码块".to_string())
                    .await?
                {
                    return Ok(timeout_flow);
                }

                let (result, flow) = self.execute_rhai_block(code, "data.rhai").await?;

                if let Some(target) = out_var.as_ref().filter(|value| !value.trim().is_empty()) {
                    self.set_runtime_var(target, result).await?;
                }

                Ok(flow)
            }
        }
    }
}
