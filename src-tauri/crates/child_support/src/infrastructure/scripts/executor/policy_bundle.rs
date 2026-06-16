impl ScriptExecutor {
    async fn load_policy_bundle(&self, step_type: &str) -> ExecuteResult<PolicyBundle> {
        let script_id = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.script_id
        };
        let snapshot = get_script_bundle_snapshot(script_id).await.ok_or_else(|| {
            Self::execute_error(
                step_type,
                format!("当前 session 中不存在脚本[{}]的 bundle", script_id),
            )
        })?;

        Ok(PolicyBundle {
            policies: Self::parse_bundle_json(step_type, "policies_json", &snapshot.policies_json)?,
            policy_groups: Self::parse_bundle_json(
                step_type,
                "policy_groups_json",
                &snapshot.policy_groups_json,
            )?,
            policy_sets: Self::parse_bundle_json(
                step_type,
                "policy_sets_json",
                &snapshot.policy_sets_json,
            )?,
            group_policies: Self::parse_bundle_json(
                step_type,
                "group_policies_json",
                &snapshot.group_policies_json,
            )?,
            set_groups: Self::parse_bundle_json(
                step_type,
                "set_groups_json",
                &snapshot.set_groups_json,
            )?,
        })
    }

    fn parse_bundle_json<T>(step_type: &str, field: &str, json: &str) -> ExecuteResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(json).map_err(|error| {
            Self::execute_error(
                step_type,
                format!("解析 bundle 字段 {} 失败: {}", field, error),
            )
        })
    }

    async fn resolve_policy_set_candidates(
        &self,
        bundle: &PolicyBundle,
        target: &[PolicySetId],
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let policy_map: HashMap<PolicyId, PolicyTable> = bundle
            .policies
            .iter()
            .cloned()
            .map(|policy| (policy.id, policy))
            .collect();
        let group_map: HashMap<PolicyGroupId, PolicyGroupTable> = bundle
            .policy_groups
            .iter()
            .cloned()
            .map(|group| (group.id, group))
            .collect();
        let set_map: HashMap<PolicySetId, PolicySetTable> = bundle
            .policy_sets
            .iter()
            .cloned()
            .map(|set| (set.id, set))
            .collect();

        let overlays = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.policy_set_overlays.clone()
        };

        let mut expanded_targets = Vec::new();
        for set_id in target {
            Self::collect_policy_set_targets(
                *set_id,
                &overlays,
                &set_map,
                &mut expanded_targets,
                &mut Vec::new(),
            )?;
        }

        let mut candidates = Vec::new();
        for set_id in &expanded_targets {
            let mut group_relations: Vec<_> = bundle
                .set_groups
                .iter()
                .filter(|relation| relation.set_id == *set_id)
                .cloned()
                .collect();
            group_relations.sort_by_key(|relation| relation.order_index);

            for group_relation in group_relations {
                if !group_map.contains_key(&group_relation.group_id) {
                    Log::warn(&format!(
                        "[ executor ] 策略集[{}]引用的策略组[{}]不存在，已跳过",
                        set_id, group_relation.group_id
                    ));
                    continue;
                }

                let mut policy_relations: Vec<_> = bundle
                    .group_policies
                    .iter()
                    .filter(|relation| relation.group_id == group_relation.group_id)
                    .cloned()
                    .collect();
                policy_relations.sort_by_key(|relation| relation.order_index);

                for policy_relation in policy_relations {
                    let Some(policy) = policy_map.get(&policy_relation.policy_id) else {
                        Log::warn(&format!(
                            "[ executor ] 策略组[{}]引用的策略[{}]不存在，已跳过",
                            group_relation.group_id, policy_relation.policy_id
                        ));
                        continue;
                    };

                    candidates.push(PolicyCandidate {
                        policy_set_id: Some(*set_id),
                        policy_set_name: set_map.get(set_id).map(|set| set.data.0.name.clone()),
                        policy_group_id: Some(group_relation.group_id),
                        policy_group_name: group_map
                            .get(&group_relation.group_id)
                            .map(|group| group.data.0.name.clone()),
                        policy: policy.clone(),
                    });
                }
            }
        }

        Ok(candidates)
    }

    fn collect_policy_set_targets(
        set_id: PolicySetId,
        overlays: &HashMap<PolicySetId, Vec<PolicySetId>>,
        set_map: &HashMap<PolicySetId, PolicySetTable>,
        output: &mut Vec<PolicySetId>,
        visiting: &mut Vec<PolicySetId>,
    ) -> ExecuteResult<()> {
        if !set_map.contains_key(&set_id) {
            return Err(Self::execute_error(
                "flow.handlePolicySet",
                format!("目标策略集[{}]不存在", set_id),
            ));
        }

        if output.contains(&set_id) {
            return Ok(());
        }
        if visiting.contains(&set_id) {
            return Err(Self::execute_error(
                "flow.handlePolicySet",
                format!("策略集追加关系存在循环: {}", set_id),
            ));
        }

        visiting.push(set_id);
        if let Some(sources) = overlays.get(&set_id) {
            for source in sources {
                Self::collect_policy_set_targets(*source, overlays, set_map, output, visiting)?;
            }
        }
        visiting.pop();

        if !output.contains(&set_id) {
            output.push(set_id);
        }

        Ok(())
    }

    fn resolve_policy_candidates(
        bundle: &PolicyBundle,
        target: &[PolicyId],
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let policy_map: HashMap<PolicyId, PolicyTable> = bundle
            .policies
            .iter()
            .cloned()
            .map(|policy| (policy.id, policy))
            .collect();

        let mut candidates = Vec::new();
        for policy_id in target {
            let Some(policy) = policy_map.get(policy_id) else {
                return Err(Self::execute_error(
                    "flow.handlePolicy",
                    format!("目标策略[{}]不存在", policy_id),
                ));
            };

            candidates.push(PolicyCandidate {
                policy_set_id: None,
                policy_set_name: None,
                policy_group_id: None,
                policy_group_name: None,
                policy: policy.clone(),
            });
        }

        Ok(candidates)
    }

    fn resolve_policy_group_candidates(
        bundle: &PolicyBundle,
        group_id: PolicyGroupId,
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let group_exists = bundle.policy_groups.iter().any(|group| group.id == group_id);
        if !group_exists {
            return Err(Self::execute_error(
                "debug.policyGroup",
                format!("目标策略组[{}]不存在", group_id),
            ));
        }

        let policy_map: HashMap<PolicyId, PolicyTable> = bundle
            .policies
            .iter()
            .cloned()
            .map(|policy| (policy.id, policy))
            .collect();
        let mut policy_relations: Vec<_> = bundle
            .group_policies
            .iter()
            .filter(|relation| relation.group_id == group_id)
            .cloned()
            .collect();
        policy_relations.sort_by_key(|relation| relation.order_index);

        let mut candidates = Vec::new();
        for policy_relation in policy_relations {
            let Some(policy) = policy_map.get(&policy_relation.policy_id) else {
                return Err(Self::execute_error(
                    "debug.policyGroup",
                    format!(
                        "策略组[{}]引用的策略[{}]不存在",
                        group_id, policy_relation.policy_id
                    ),
                ));
            };

            candidates.push(PolicyCandidate {
                policy_set_id: None,
                policy_set_name: None,
                policy_group_id: Some(group_id),
                policy_group_name: bundle
                    .policy_groups
                    .iter()
                    .find(|group| group.id == group_id)
                    .map(|group| group.data.0.name.clone()),
                policy: policy.clone(),
            });
        }

        Ok(candidates)
    }
}
