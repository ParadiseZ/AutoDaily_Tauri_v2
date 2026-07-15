impl ScriptExecutor {
    async fn resolve_step_display_name(&self, step: &Step) -> String {
        let base_name = step
            .label
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("未命名步骤");

        let StepKind::FlowControl {
            a: FlowControl::HandlePolicySet { target, .. },
        } = &step.kind
        else {
            return base_name.to_string();
        };

        let Ok(bundle) = self.load_policy_bundle("executor.stepDisplay").await else {
            return base_name.to_string();
        };

        let set_names = target
            .iter()
            .filter_map(|set_id| {
                bundle
                    .policy_sets
                    .iter()
                    .find(|set| set.id == *set_id)
                    .map(|set| set.info.name.trim().to_string())
                    .filter(|name| !name.is_empty())
            })
            .collect::<Vec<_>>();

        if set_names.is_empty() {
            return base_name.to_string();
        }

        format!("{}-{}", base_name, set_names.join("/"))
    }

    pub(crate) async fn debug_execute_policy(
        &mut self,
        policy_id: PolicyId,
    ) -> ExecuteResult<PolicyExecutionResult> {
        let bundle = self.load_policy_bundle("debug.policy").await?;
        let candidates = Self::resolve_policy_candidates(&bundle, &[policy_id])?;
        self.debug_execute_policy_candidates("策略", candidates).await
    }

    pub(crate) async fn debug_execute_policy_group(
        &mut self,
        policy_group_id: PolicyGroupId,
    ) -> ExecuteResult<PolicyExecutionResult> {
        let bundle = self.load_policy_bundle("debug.policyGroup").await?;
        let candidates = self
            .resolve_policy_group_candidates(&bundle, policy_group_id, "debug.policyGroup")
            .await?;
        self.debug_execute_policy_candidates("策略组", candidates).await
    }

    pub(crate) async fn debug_execute_policy_set(
        &mut self,
        policy_set_id: PolicySetId,
    ) -> ExecuteResult<PolicyExecutionResult> {
        let bundle = self.load_policy_bundle("debug.policySet").await?;
        let candidates = self
            .resolve_policy_set_candidates(&bundle, &[policy_set_id])
            .await?;
        self.debug_execute_policy_candidates("策略集", candidates).await
    }

}
