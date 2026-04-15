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

    fn state_target_label(target: &StateTarget) -> String {
        match target {
            StateTarget::Task { id } => format!("task:{}", id),
            StateTarget::Policy { id } => format!("policy:{}", id),
        }
    }

    fn execute_error(step_type: &str, e: String) -> ScriptError {
        ScriptError::ExecuteErr {
            step_type: step_type.to_string(),
            e,
        }
    }
}
