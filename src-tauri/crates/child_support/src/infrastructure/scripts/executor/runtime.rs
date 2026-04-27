impl ScriptExecutor {
    async fn set_runtime_var(&mut self, name: &str, value: Dynamic) -> ExecuteResult<()> {
        if name.trim().is_empty() {
            return Ok(());
        }

        let root = name.split('.').next().unwrap_or(name).trim().to_string();
        let root_value = {
            let mut ctx = self.runtime_ctx.write().await;
            ctx.execution.var_map.insert(name.to_string(), value);
            Self::build_scope_root_value(&ctx.execution.var_map, &root)
        };
        self.scope.set_value(root, root_value);
        Ok(())
    }

    async fn read_runtime_var(&self, name: &str) -> Option<Dynamic> {
        {
            let ctx = self.runtime_ctx.read().await;
            if let Some(value) = ctx.execution.var_map.get(name) {
                return Some(value.clone());
            }
        }

        if name.contains('.') {
            None
        } else {
            self.scope.get_value::<Dynamic>(name)
        }
    }

    async fn remove_runtime_var(&mut self, name: &str) {
        if name.trim().is_empty() {
            return;
        }

        let root = name.split('.').next().unwrap_or(name).trim().to_string();
        let root_value = {
            let mut ctx = self.runtime_ctx.write().await;
            ctx.execution.var_map.remove(name);
            Self::build_scope_root_value(&ctx.execution.var_map, &root)
        };
        self.scope.set_value(root, root_value);
    }

    pub(crate) async fn hydrate_input_scope(
        &mut self,
        variable_catalog: &ScriptVariableCatalog,
        template_values_json: Option<&str>,
        task: Option<&ScriptTaskTable>,
    ) -> ExecuteResult<()> {
        let effective_template_values_json = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution
                .template_values_json
                .clone()
                .or_else(|| template_values_json.map(str::to_string))
        };
        let template_values =
            Self::parse_runtime_template_values(effective_template_values_json.as_deref())?;
        self.clear_input_scope(variable_catalog).await;

        let run_target = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.target.clone()
        };

        for variable in variable_catalog
            .variables
            .iter()
            .filter(|variable| matches!(variable.namespace, ScriptVariableNamespace::Input))
        {
            if !Self::input_variable_visible_for_target(variable, &run_target, task) {
                continue;
            }

            let value = Self::resolve_input_variable_value(variable, template_values.as_ref(), task);
            match value {
                Some(value) => {
                    let dynamic = to_dynamic(&value).map_err(|error| {
                        Self::execute_error(
                            "runtime.inputScope",
                            format!("输入变量[{}]装入运行时失败: {}", variable.key, error),
                        )
                    })?;
                    self.set_runtime_var(&variable.key, dynamic).await?;
                }
                None => {
                    let prefix = if matches!(run_target, RunTarget::DeviceQueue) {
                        "[ runtime ]"
                    } else {
                        "【调试】"
                    };
                    Log::debug(&format!(
                        "{} 未取到目标值: variable={}, variableId={}",
                        prefix, variable.key, variable.id
                    ));
                }
            }
        }

        Ok(())
    }

    async fn clear_input_scope(&mut self, variable_catalog: &ScriptVariableCatalog) {
        for variable in variable_catalog
            .variables
            .iter()
            .filter(|variable| matches!(variable.namespace, ScriptVariableNamespace::Input))
        {
            self.remove_runtime_var(&variable.key).await;
        }
    }

    fn parse_runtime_template_values(
        json: Option<&str>,
    ) -> ExecuteResult<Option<RuntimeTemplateValuesSnapshot>> {
        match json {
            Some(content) if !content.trim().is_empty() && content.trim() != "null" => {
                serde_json::from_str(content).map(Some).map_err(|error| {
                    Self::execute_error(
                        "runtime.inputScope",
                        format!("解析模板覆盖值失败: {}", error),
                    )
                })
            }
            _ => Ok(None),
        }
    }

    fn input_variable_visible_for_task(
        variable: &ScriptVariableDef,
        task: Option<&ScriptTaskTable>,
    ) -> bool {
        match (variable.owner_task_id, task.map(|task| task.id)) {
            (None, _) => true,
            (Some(owner_task_id), Some(task_id)) => owner_task_id == task_id,
            (Some(_), None) => false,
        }
    }

    fn input_variable_visible_for_target(
        variable: &ScriptVariableDef,
        run_target: &RunTarget,
        task: Option<&ScriptTaskTable>,
    ) -> bool {
        if Self::input_variable_visible_for_task(variable, task) {
            return true;
        }

        matches!(
            run_target,
            RunTarget::Policy { .. } | RunTarget::PolicyGroup { .. } | RunTarget::PolicySet { .. }
        ) && task.is_none()
            && variable.owner_task_id.is_some()
    }

    fn resolve_input_variable_value(
        variable: &ScriptVariableDef,
        template_values: Option<&RuntimeTemplateValuesSnapshot>,
        task: Option<&ScriptTaskTable>,
    ) -> Option<serde_json::Value> {
        template_values
            .and_then(|snapshot| snapshot.variables.get(&variable.id).cloned())
            .or_else(|| Self::resolve_task_default_variable_value(variable, task))
            .or_else(|| variable.default_value.clone())
    }

    fn resolve_task_default_variable_value(
        variable: &ScriptVariableDef,
        task: Option<&ScriptTaskTable>,
    ) -> Option<serde_json::Value> {
        let task = task?;
        let key = Self::input_variable_storage_key(variable);
        task.data
            .0
            .variables
            .as_object()
            .and_then(|variables| variables.get(key).cloned())
    }

    fn input_variable_storage_key(variable: &ScriptVariableDef) -> &str {
        variable
            .key
            .strip_prefix("input.")
            .unwrap_or(variable.key.as_str())
    }

    fn build_scope_root_value(var_map: &HashMap<String, Dynamic>, root: &str) -> Dynamic {
        let nested_prefix = format!("{}.", root);
        let mut nested = Map::new();

        for (key, value) in var_map {
            if let Some(suffix) = key.strip_prefix(&nested_prefix) {
                let partial = Self::build_nested_map(suffix, value.clone());
                Self::merge_map(&mut nested, partial);
            }
        }

        if nested.is_empty() {
            var_map.get(root).cloned().unwrap_or(Dynamic::UNIT)
        } else {
            Dynamic::from(nested)
        }
    }

    fn build_nested_map(path: &str, value: Dynamic) -> Map {
        let mut current = value;
        for segment in path.split('.').rev() {
            let mut map = Map::new();
            map.insert(segment.into(), current);
            current = Dynamic::from(map);
        }

        current.try_cast::<Map>().unwrap_or_default()
    }

    fn merge_map(target: &mut Map, source: Map) {
        for (key, value) in source {
            if let Some(existing) = target.get_mut(&key) {
                let left = existing.clone().try_cast::<Map>();
                let right = value.clone().try_cast::<Map>();
                match (left, right) {
                    (Some(mut left_map), Some(right_map)) => {
                        Self::merge_map(&mut left_map, right_map);
                        *existing = Dynamic::from(left_map);
                    }
                    _ => *existing = value,
                }
            } else {
                target.insert(key, value);
            }
        }
    }

    fn var_value_to_dynamic(value: &VarValue) -> Dynamic {
        match value {
            VarValue::Int(value) => Dynamic::from_int((*value).into()),
            VarValue::Float(value) => Dynamic::from_float((*value).into()),
            VarValue::Bool(value) => Dynamic::from_bool(*value),
            VarValue::String(value) => Dynamic::from(value.clone()),
        }
    }

    fn search_hits_to_dynamic(hits: &[SearchHit]) -> Dynamic {
        let mut array = Array::new();
        for hit in hits {
            let mut item = Map::new();
            item.insert("pattern".into(), Dynamic::from(hit.pattern.clone()));
            item.insert("ocrIndex".into(), Dynamic::from_int(hit.ocr_index as INT));
            item.insert("text".into(), Dynamic::from(hit.ocr_item.txt.clone()));
            array.push(Dynamic::from(item));
        }
        Dynamic::from(array)
    }

    fn compare_dynamic(lhs: &Dynamic, op: &CompareOp, rhs: &Dynamic) -> bool {
        match op {
            CompareOp::Contains => Self::dynamic_to_string(lhs)
                .zip(Self::dynamic_to_string(rhs))
                .map(|(lhs, rhs)| lhs.contains(&rhs))
                .unwrap_or(false),
            CompareOp::NotContains => Self::dynamic_to_string(lhs)
                .zip(Self::dynamic_to_string(rhs))
                .map(|(lhs, rhs)| !lhs.contains(&rhs))
                .unwrap_or(false),
            CompareOp::Eq => Self::dynamic_eq(lhs, rhs),
            CompareOp::Ne => !Self::dynamic_eq(lhs, rhs),
            CompareOp::Lt => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs < rhs)
                .unwrap_or(false),
            CompareOp::Le => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs <= rhs)
                .unwrap_or(false),
            CompareOp::Gt => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs > rhs)
                .unwrap_or(false),
            CompareOp::Ge => Self::dynamic_to_number(lhs)
                .zip(Self::dynamic_to_number(rhs))
                .map(|(lhs, rhs)| lhs >= rhs)
                .unwrap_or(false),
        }
    }

    fn compare_bool(actual: bool, op: &PolicySetResultCompareOp, expected: bool) -> bool {
        match op {
            PolicySetResultCompareOp::Eq => actual == expected,
            PolicySetResultCompareOp::Ne => actual != expected,
        }
    }

    fn compare_optional_id<T>(
        actual: Option<T>,
        op: &PolicySetResultCompareOp,
        expected: &str,
    ) -> bool
    where
        T: Serialize,
    {
        let actual = actual
            .and_then(|value| serde_json::to_value(value).ok())
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_default();
        match op {
            PolicySetResultCompareOp::Eq => actual == expected,
            PolicySetResultCompareOp::Ne => actual != expected,
        }
    }

    fn deserialize_dynamic_value<T>(value: &Dynamic) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        match from_dynamic::<T>(value) {
            Ok(parsed) => Ok(parsed),
            Err(primary_error) => {
                let primary_message = primary_error.to_string();
                let json_value = from_dynamic::<serde_json::Value>(value).map_err(|json_error| {
                    format!("{}；JSON 中转失败: {}", primary_message, json_error)
                })?;

                serde_json::from_value(json_value).map_err(|json_error| {
                    format!("{}；JSON 回退反序列化失败: {}", primary_message, json_error)
                })
            }
        }
    }

    fn dynamic_eq(lhs: &Dynamic, rhs: &Dynamic) -> bool {
        if let (Some(lhs), Some(rhs)) = (
            lhs.clone().try_cast::<bool>(),
            rhs.clone().try_cast::<bool>(),
        ) {
            return lhs == rhs;
        }
        if let (Some(lhs), Some(rhs)) = (Self::dynamic_to_number(lhs), Self::dynamic_to_number(rhs))
        {
            return (lhs - rhs).abs() < f64::EPSILON;
        }
        if let (Some(lhs), Some(rhs)) = (Self::dynamic_to_string(lhs), Self::dynamic_to_string(rhs))
        {
            return lhs == rhs;
        }
        false
    }

    fn dynamic_to_number(value: &Dynamic) -> Option<f64> {
        if let Some(value) = value.clone().try_cast::<INT>() {
            return Some(value as f64);
        }
        if let Some(value) = value.clone().try_cast::<FLOAT>() {
            return Some(value as f64);
        }
        if let Some(value) = value.clone().try_cast::<String>() {
            return value.parse::<f64>().ok();
        }
        None
    }

    fn dynamic_to_string(value: &Dynamic) -> Option<String> {
        if let Some(value) = value.clone().try_cast::<String>() {
            return Some(value);
        }
        if let Some(value) = value.clone().try_cast::<bool>() {
            return Some(value.to_string());
        }
        if let Some(value) = value.clone().try_cast::<INT>() {
            return Some(value.to_string());
        }
        if let Some(value) = value.clone().try_cast::<FLOAT>() {
            return Some(value.to_string());
        }
        None
    }

    fn eval_bool(&mut self, expr: &str, step_type: &str) -> ExecuteResult<bool> {
        self.engine
            .eval_expression_with_scope::<bool>(&mut self.scope, expr)
            .map_err(|error| Self::execute_error(step_type, error.to_string()))
    }

    fn eval_dynamic(&mut self, expr: &str, step_type: &str) -> ExecuteResult<Dynamic> {
        self.engine
            .eval_expression_with_scope::<Dynamic>(&mut self.scope, expr)
            .map_err(|error| Self::execute_error(step_type, error.to_string()))
    }

    fn eval_repeat_count(&mut self, expr: &str, step_type: &str) -> ExecuteResult<usize> {
        let value = self.eval_dynamic(expr, step_type)?;
        let count = if let Some(value) = value.clone().try_cast::<INT>() {
            value
        } else if let Some(value) = value.clone().try_cast::<FLOAT>() {
            value.floor() as INT
        } else if let Some(value) = value.clone().try_cast::<String>() {
            value.trim().parse::<INT>().map_err(|error| {
                Self::execute_error(
                    step_type,
                    format!("循环次数表达式结果无法转为整数: {}", error),
                )
            })?
        } else {
            return Err(Self::execute_error(
                step_type,
                "循环次数表达式必须返回数字或数字字符串".to_string(),
            ));
        };

        if count <= 0 {
            return Ok(0);
        }
        usize::try_from(count).map_err(|_| {
            Self::execute_error(
                step_type,
                format!("循环次数超出可支持范围: {}", count),
            )
        })
    }

    fn execute_error(step_type: &str, e: String) -> ScriptError {
        ScriptError::ExecuteErr {
            step_type: step_type.to_string(),
            e,
        }
    }
}
