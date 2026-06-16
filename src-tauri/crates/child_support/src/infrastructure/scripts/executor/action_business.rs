impl ScriptExecutor {
    async fn adjust_policy_click_pos(&self, policy_id: PolicyId, delta: i16) -> ExecuteResult<()> {
        let base = self.resolve_policy_base_click_pos(policy_id).await?;
        let mut ctx = self.runtime_ctx.write().await;
        let state = ctx.execution.policy_states.entry(policy_id).or_default();
        let current = state.click_pos.unwrap_or(base);
        state.click_pos = Some(if delta >= 0 {
            current.saturating_add(delta as u16)
        } else {
            current.saturating_sub(delta.unsigned_abs())
        });
        Ok(())
    }

    async fn resolve_policy_base_click_pos(&self, policy_id: PolicyId) -> ExecuteResult<u16> {
        if let Some(active) = self
            .active_policy_context
            .as_ref()
            .filter(|context| context.policy_id == policy_id)
        {
            return Ok(active.base_click_pos);
        }

        let bundle = self.load_policy_bundle("action.policyPosition").await?;
        bundle
            .policies
            .into_iter()
            .find(|policy| policy.id == policy_id)
            .map(|policy| policy.data.0.cur_pos)
            .ok_or_else(|| {
                Self::execute_error(
                    "action.policyPosition",
                    format!("目标策略[{}]不存在，无法调整点击索引", policy_id),
                )
            })
    }

    async fn execute_drop_set_next(
        &mut self,
        task_id: TaskId,
        variable_id: &str,
    ) -> ExecuteResult<()> {
        let variable_id = variable_id.trim();
        if variable_id.is_empty() {
            return Err(Self::execute_error(
                "action.dropSetNext",
                "DropSetNext 需要选择目标 UI 变量".to_string(),
            ));
        }

        let script_id = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.script_id
        };
        let snapshot = get_script_bundle_snapshot(script_id).await.ok_or_else(|| {
            Self::execute_error("action.dropSetNext", "当前 session 中不存在脚本 bundle".to_string())
        })?;
        let script: ScriptTable =
            Self::parse_bundle_json("action.dropSetNext", "script_json", &snapshot.script_json)?;
        let tasks: Vec<ScriptTaskTable> =
            Self::parse_bundle_json("action.dropSetNext", "tasks_json", &snapshot.tasks_json)?;
        let task = tasks.into_iter().find(|task| task.id == task_id).ok_or_else(|| {
            Self::execute_error(
                "action.dropSetNext",
                format!("目标任务[{}]不存在", task_id),
            )
        })?;
        let variable = script
            .data
            .0
            .variable_catalog
            .variables
            .iter()
            .find(|variable| variable.id == variable_id)
            .ok_or_else(|| {
                Self::execute_error(
                    "action.dropSetNext",
                    format!("目标变量[{}]不存在", variable_id),
                )
            })?;
        if !matches!(variable.namespace, ScriptVariableNamespace::Input) || !variable.persisted {
            return Err(Self::execute_error(
                "action.dropSetNext",
                format!("目标变量[{}]不是可持久化 Input 变量", variable_id),
            ));
        }

        let options = Self::resolve_task_ui_options_for_variable(&task, variable_id)?;
        let next_value = {
            let ctx = self.runtime_ctx.read().await;
            let root = Self::parse_template_values_root(ctx.execution.template_values_json.as_deref())?;
            let current = Self::template_variable_value(&root, variable_id)
                .or_else(|| variable.default_value.as_ref().map(Self::json_value_to_string));
            Self::next_option_value(&options, current.as_deref())
        };

        let (device_id, time_template_id, account_id, next_root_json) = {
            let mut ctx = self.runtime_ctx.write().await;
            let device_id = ctx.execution.current_device_id.ok_or_else(|| {
                Self::execute_error("action.dropSetNext", "当前运行时缺少设备 ID".to_string())
            })?;
            let time_template_id = ctx.execution.current_time_template_id.ok_or_else(|| {
                Self::execute_error(
                    "action.dropSetNext",
                    "当前运行没有时间模板，无法持久化 UI 变量切换".to_string(),
                )
            })?;
            let account_id = ctx.execution.current_account_id.clone();
            let mut root = Self::parse_template_values_root(ctx.execution.template_values_json.as_deref())?;
            Self::set_template_variable_value(&mut root, variable_id, Value::String(next_value.clone()));
            let root_json = serde_json::to_string(&root).map_err(|error| {
                Self::execute_error(
                    "action.dropSetNext",
                    format!("序列化模板变量失败: {}", error),
                )
            })?;
            ctx.execution.template_values_json = Some(root_json.clone());
            (device_id, time_template_id, account_id, root_json)
        };

        self.set_runtime_var(&variable.key, Dynamic::from(next_value.clone()))
            .await?;

        let next_root: Value = serde_json::from_str(&next_root_json).map_err(|error| {
            Self::execute_error(
                "action.dropSetNext",
                format!("解析更新后的模板变量失败: {}", error),
            )
        })?;
        Self::save_runtime_template_values(
            device_id,
            script_id,
            time_template_id,
            account_id,
            &next_root,
        )
        .await?;

        Log::info(&format!(
            "[ executor ] DropSetNext 已切换任务[{}]变量[{}]到 {}",
            task_id, variable_id, next_value
        ));
        Ok(())
    }

    fn resolve_task_ui_options_for_variable(
        task: &ScriptTaskTable,
        variable_id: &str,
    ) -> ExecuteResult<Vec<String>> {
        let fields = task
            .data
            .0
            .ui_data
            .get("fields")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                Self::execute_error(
                    "action.dropSetNext",
                    format!("任务[{}]没有可用 UI 字段", task.id),
                )
            })?;

        for field in fields {
            let control = field.get("control").and_then(Value::as_str).unwrap_or_default();
            if control != "select" && control != "radio" {
                continue;
            }
            if field.get("variableId").and_then(Value::as_str) != Some(variable_id) {
                continue;
            }
            let options = field
                .get("options")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| {
                            item.as_str()
                                .map(str::to_string)
                                .or_else(|| item.get("label").and_then(Value::as_str).map(str::to_string))
                        })
                        .filter(|item| !item.trim().is_empty())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if options.is_empty() {
                return Err(Self::execute_error(
                    "action.dropSetNext",
                    format!("任务[{}]的目标 UI 字段没有可迭代选项", task.id),
                ));
            }
            return Ok(options);
        }

        Err(Self::execute_error(
            "action.dropSetNext",
            format!("任务[{}]没有绑定变量[{}]的 select/radio 字段", task.id, variable_id),
        ))
    }

    fn parse_template_values_root(json: Option<&str>) -> ExecuteResult<Value> {
        match json {
            Some(content) if !content.trim().is_empty() && content.trim() != "null" => {
                serde_json::from_str(content).map_err(|error| {
                    Self::execute_error(
                        "action.dropSetNext",
                        format!("解析模板变量快照失败: {}", error),
                    )
                })
            }
            _ => Ok(json!({})),
        }
    }

    fn template_variable_value(root: &Value, variable_id: &str) -> Option<String> {
        root.get("variables")
            .and_then(Value::as_object)
            .and_then(|variables| variables.get(variable_id))
            .map(Self::json_value_to_string)
    }

    fn json_value_to_string(value: &Value) -> String {
        value
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| value.to_string())
    }

    fn next_option_value(options: &[String], current: Option<&str>) -> String {
        let current = current.map(str::trim).filter(|value| !value.is_empty());
        let index = current
            .and_then(|value| options.iter().position(|option| option == value))
            .map(|index| (index + 1) % options.len())
            .unwrap_or(0);
        options[index].clone()
    }

    fn set_template_variable_value(root: &mut Value, variable_id: &str, value: Value) {
        if !root.is_object() {
            *root = json!({});
        }
        let object = root.as_object_mut().expect("root was normalized to object");
        let variables = object
            .entry("variables".to_string())
            .or_insert_with(|| Value::Object(JsonMap::new()));
        if !variables.is_object() {
            *variables = Value::Object(JsonMap::new());
        }
        variables
            .as_object_mut()
            .expect("variables was normalized to object")
            .insert(variable_id.to_string(), value);
    }

    async fn save_runtime_template_values(
        device_id: DeviceId,
        script_id: ScriptId,
        time_template_id: TemplateId,
        account_id: Option<AccountId>,
        values_json: &Value,
    ) -> ExecuteResult<()> {
        let pool = get_pool();
        let account_id = account_id.and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        });
        let existing_id: Option<String> = sqlx::query_scalar(&format!(
            "SELECT id FROM {}
             WHERE device_id = ?
               AND script_id = ?
               AND time_template_id = ?
               AND ((account_id IS NULL AND ? IS NULL) OR account_id = ?)
             LIMIT 1",
            SCRIPT_TIME_TEMPLATE_VALUES_TABLE
        ))
        .bind(device_id.to_string())
        .bind(script_id.to_string())
        .bind(time_template_id.to_string())
        .bind(account_id.clone())
        .bind(account_id.clone())
        .fetch_optional(pool)
        .await
        .map_err(|error| Self::execute_error("action.dropSetNext", error.to_string()))?;

        let now = chrono::Local::now().to_rfc3339();
        match existing_id {
            Some(id) => {
                sqlx::query(&format!(
                    "UPDATE {} SET values_json = ?, updated_at = ? WHERE id = ?",
                    SCRIPT_TIME_TEMPLATE_VALUES_TABLE
                ))
                .bind(SqlJson(values_json.clone()))
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await
                .map_err(|error| Self::execute_error("action.dropSetNext", error.to_string()))?;
            }
            None => {
                sqlx::query(&format!(
                    "INSERT INTO {} (id, device_id, script_id, time_template_id, account_id, values_json, created_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    SCRIPT_TIME_TEMPLATE_VALUES_TABLE
                ))
                .bind(ScriptTemplateValueId::new_v7().to_string())
                .bind(device_id.to_string())
                .bind(script_id.to_string())
                .bind(time_template_id.to_string())
                .bind(account_id)
                .bind(SqlJson(values_json.clone()))
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await
                .map_err(|error| Self::execute_error("action.dropSetNext", error.to_string()))?;
            }
        }

        Ok(())
    }
}
