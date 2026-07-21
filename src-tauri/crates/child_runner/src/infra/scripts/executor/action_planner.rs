struct PlannedDeviceAction {
    operations: Vec<DeviceOperation>,
    trace: Option<PolicyActionTrace>,
}

struct ResolvedPrimaryTargets {
    points: Vec<Point<u16>>,
    source: PolicyActionSource,
    targets: Vec<PolicyActionTarget>,
}

impl ScriptExecutor {
    async fn execute_planned_device_action(
        &self,
        step_type: &str,
        label: &str,
        plan: PlannedDeviceAction,
    ) -> ExecuteResult<(ControlFlow, Option<PolicyActionTrace>)> {
        if plan.operations.is_empty() {
            return Ok((ControlFlow::Next, plan.trace));
        }

        Self::await_device_result_with_timeout(
            step_type,
            label,
            DEVICE_EXTERNAL_TIMEOUT_MS,
            self.execute_device_operations(&plan.operations),
        )
        .await?;

        Ok((ControlFlow::Next, plan.trace))
    }
}
