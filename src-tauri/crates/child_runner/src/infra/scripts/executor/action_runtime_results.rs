impl ScriptExecutor {
    async fn read_runtime_result_vec<T>(
        &self,
        input_var: &str,
        step_type: &str,
        result_label: &str,
    ) -> ExecuteResult<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let value = self.read_runtime_var(input_var).await.ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!(
                    "输入变量[{}]不存在，无法读取{}结果集",
                    input_var, result_label
                ),
            )
        })?;

        Self::deserialize_dynamic_value::<Vec<T>>(&value).map_err(|error| {
            Self::execute_error(
                step_type,
                format!(
                    "输入变量[{}]不是兼容的{}结果集，无法执行动作: {}",
                    input_var, result_label, error
                ),
            )
        })
    }
}
