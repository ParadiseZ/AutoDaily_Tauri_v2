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
        let set_group_map = Self::build_set_group_map(&bundle.set_groups);
        let group_policy_map = Self::build_group_policy_map(&bundle.group_policies);

        let (set_bindings, group_bindings) = {
            let ctx = self.runtime_ctx.read().await;
            (
                ctx.execution.policy_set_bindings.clone(),
                ctx.execution.policy_group_bindings.clone(),
            )
        };

        let mut candidates = Vec::new();
        for set_id in target {
            let group_ids = Self::resolve_policy_set_group_ids(
                "flow.handlePolicySet",
                *set_id,
                &set_map,
                &set_group_map,
                &set_bindings,
                &mut Vec::new(),
            )?;
            Log::debug(&format!(
                "[ executor ] 策略集[{}]展开后的策略组顺序: {}",
                set_id,
                Self::format_id_list(&group_ids)
            ));

            for group_id in group_ids {
                let policy_ids = Self::resolve_policy_group_policy_ids(
                    "flow.handlePolicySet",
                    group_id,
                    &group_map,
                    &group_policy_map,
                    &group_bindings,
                    &mut Vec::new(),
                )?;
                Log::debug(&format!(
                    "[ executor ] 策略组[{}]展开后的策略顺序: {}",
                    group_id,
                    Self::format_id_list(&policy_ids)
                ));

                for policy_id in policy_ids {
                    let Some(policy) = policy_map.get(&policy_id) else {
                        Log::warn(&format!(
                            "[ executor ] 策略组[{}]引用的策略[{}]不存在，已跳过",
                            group_id, policy_id
                        ));
                        continue;
                    };

                    candidates.push(PolicyCandidate {
                        policy_set_id: Some(*set_id),
                        policy_set_name: set_map.get(set_id).map(|set| set.data.0.name.clone()),
                        policy_group_id: Some(group_id),
                        policy_group_name: group_map
                            .get(&group_id)
                            .map(|group| group.data.0.name.clone()),
                        policy: policy.clone(),
                    });
                }
            }
        }

        Ok(candidates)
    }

    fn resolve_policy_set_group_ids(
        step_type: &str,
        set_id: PolicySetId,
        set_map: &HashMap<PolicySetId, PolicySetTable>,
        set_group_map: &HashMap<PolicySetId, Vec<PolicyGroupId>>,
        set_bindings: &HashMap<PolicySetId, Vec<PolicySetBindingOp>>,
        visiting: &mut Vec<PolicySetId>,
    ) -> ExecuteResult<Vec<PolicyGroupId>> {
        if !set_map.contains_key(&set_id) {
            return Err(Self::execute_error(
                step_type,
                format!("目标策略集[{}]不存在", set_id),
            ));
        }
        if visiting.contains(&set_id) {
            let mut chain = visiting.iter().map(ToString::to_string).collect::<Vec<_>>();
            chain.push(set_id.to_string());
            return Err(Self::execute_error(
                step_type,
                format!("策略集绑定存在循环: {}", chain.join(" -> ")),
            ));
        }

        visiting.push(set_id);
        let mut group_ids = set_group_map.get(&set_id).cloned().unwrap_or_default();
        if let Some(bindings) = set_bindings.get(&set_id) {
            for binding in bindings {
                let source_ids = match binding.source {
                    PolicySetBindingSource::PolicySet(source_set_id) => Self::resolve_policy_set_group_ids(
                        step_type,
                        source_set_id,
                        set_map,
                        set_group_map,
                        set_bindings,
                        visiting,
                    )?,
                    PolicySetBindingSource::PolicyGroup(source_group_id) => vec![source_group_id],
                };
                Self::merge_bound_items(&mut group_ids, source_ids, binding.top, binding.reverse);
            }
        }
        visiting.pop();

        Ok(group_ids)
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

    async fn resolve_policy_group_candidates(
        &self,
        bundle: &PolicyBundle,
        group_id: PolicyGroupId,
        step_type: &str,
    ) -> ExecuteResult<Vec<PolicyCandidate>> {
        let group_map: HashMap<PolicyGroupId, PolicyGroupTable> = bundle
            .policy_groups
            .iter()
            .cloned()
            .map(|group| (group.id, group))
            .collect();
        let policy_map: HashMap<PolicyId, PolicyTable> = bundle
            .policies
            .iter()
            .cloned()
            .map(|policy| (policy.id, policy))
            .collect();
        let group_policy_map = Self::build_group_policy_map(&bundle.group_policies);
        let group_bindings = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution.policy_group_bindings.clone()
        };

        let policy_ids = Self::resolve_policy_group_policy_ids(
            step_type,
            group_id,
            &group_map,
            &group_policy_map,
            &group_bindings,
            &mut Vec::new(),
        )?;
        Log::debug(&format!(
            "[ executor ] 策略组[{}]展开后的策略顺序: {}",
            group_id,
            Self::format_id_list(&policy_ids)
        ));

        let mut candidates = Vec::new();
        for policy_id in policy_ids {
            let Some(policy) = policy_map.get(&policy_id) else {
                return Err(Self::execute_error(
                    step_type,
                    format!("策略组[{}]引用的策略[{}]不存在", group_id, policy_id),
                ));
            };

            candidates.push(PolicyCandidate {
                policy_set_id: None,
                policy_set_name: None,
                policy_group_id: Some(group_id),
                policy_group_name: group_map
                    .get(&group_id)
                    .map(|group| group.data.0.name.clone()),
                policy: policy.clone(),
            });
        }

        Ok(candidates)
    }

    fn resolve_policy_group_policy_ids(
        step_type: &str,
        group_id: PolicyGroupId,
        group_map: &HashMap<PolicyGroupId, PolicyGroupTable>,
        group_policy_map: &HashMap<PolicyGroupId, Vec<PolicyId>>,
        group_bindings: &HashMap<PolicyGroupId, Vec<PolicyGroupBindingOp>>,
        visiting: &mut Vec<PolicyGroupId>,
    ) -> ExecuteResult<Vec<PolicyId>> {
        if !group_map.contains_key(&group_id) {
            return Err(Self::execute_error(
                step_type,
                format!("目标策略组[{}]不存在", group_id),
            ));
        }
        if visiting.contains(&group_id) {
            let mut chain = visiting.iter().map(ToString::to_string).collect::<Vec<_>>();
            chain.push(group_id.to_string());
            return Err(Self::execute_error(
                step_type,
                format!("策略组绑定存在循环: {}", chain.join(" -> ")),
            ));
        }

        visiting.push(group_id);
        let mut policy_ids = group_policy_map.get(&group_id).cloned().unwrap_or_default();
        if let Some(bindings) = group_bindings.get(&group_id) {
            for binding in bindings {
                let source_ids = match binding.source {
                    PolicyGroupBindingSource::Policy(source_policy_id) => vec![source_policy_id],
                    PolicyGroupBindingSource::PolicyGroup(source_group_id) => {
                        Self::resolve_policy_group_policy_ids(
                            step_type,
                            source_group_id,
                            group_map,
                            group_policy_map,
                            group_bindings,
                            visiting,
                        )?
                    }
                };
                Self::merge_bound_items(&mut policy_ids, source_ids, binding.top, binding.reverse);
            }
        }
        visiting.pop();

        Ok(policy_ids)
    }

    fn build_set_group_map(
        relations: &[SetGroupRelation],
    ) -> HashMap<PolicySetId, Vec<PolicyGroupId>> {
        let mut relation_map: HashMap<PolicySetId, Vec<SetGroupRelation>> = HashMap::new();
        for relation in relations {
            relation_map
                .entry(relation.set_id)
                .or_default()
                .push(relation.clone());
        }

        relation_map
            .into_iter()
            .map(|(set_id, mut items)| {
                items.sort_by_key(|relation| relation.order_index);
                (
                    set_id,
                    items.into_iter().map(|relation| relation.group_id).collect(),
                )
            })
            .collect()
    }

    fn build_group_policy_map(
        relations: &[GroupPolicyRelation],
    ) -> HashMap<PolicyGroupId, Vec<PolicyId>> {
        let mut relation_map: HashMap<PolicyGroupId, Vec<GroupPolicyRelation>> = HashMap::new();
        for relation in relations {
            relation_map
                .entry(relation.group_id)
                .or_default()
                .push(relation.clone());
        }

        relation_map
            .into_iter()
            .map(|(group_id, mut items)| {
                items.sort_by_key(|relation| relation.order_index);
                (
                    group_id,
                    items.into_iter().map(|relation| relation.policy_id).collect(),
                )
            })
            .collect()
    }

    fn merge_bound_items<T>(target: &mut Vec<T>, mut source: Vec<T>, top: bool, reverse: bool)
    where
        T: Copy,
    {
        if reverse {
            source.reverse();
        }
        if top {
            source.extend(target.iter().copied());
            *target = source;
            return;
        }
        target.extend(source);
    }

    fn format_id_list<T>(items: &[T]) -> String
    where
        T: ToString,
    {
        if items.is_empty() {
            return "<empty>".to_string();
        }
        items.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

#[cfg(test)]
mod policy_binding_resolution_tests {
    use super::*;

    fn set_id() -> PolicySetId {
        PolicySetId::new_v7()
    }

    fn group_id() -> PolicyGroupId {
        PolicyGroupId::new_v7()
    }

    fn policy_id() -> PolicyId {
        PolicyId::new_v7()
    }

    #[test]
    fn resolves_policy_set_bindings_with_top_and_reverse() {
        let set_a = set_id();
        let set_b = set_id();
        let group_a = group_id();
        let group_b = group_id();
        let group_c = group_id();
        let group_d = group_id();

        let mut set_map = HashMap::new();
        set_map.insert(
            set_a,
            PolicySetTable {
                id: set_a,
                script_id: ScriptId::new_v7(),
                order_index: 0,
                data: SqlJson(crate::domain::scripts::policy::PolicySetInfo {
                    name: "A".to_string(),
                    note: String::new(),
                }),
            },
        );
        set_map.insert(
            set_b,
            PolicySetTable {
                id: set_b,
                script_id: ScriptId::new_v7(),
                order_index: 1,
                data: SqlJson(crate::domain::scripts::policy::PolicySetInfo {
                    name: "B".to_string(),
                    note: String::new(),
                }),
            },
        );

        let mut set_group_map = HashMap::new();
        set_group_map.insert(set_a, vec![group_a, group_b]);
        set_group_map.insert(set_b, vec![group_c]);

        let mut bindings = HashMap::new();
        bindings.insert(
            set_a,
            vec![
                PolicySetBindingOp {
                    source: PolicySetBindingSource::PolicyGroup(group_d),
                    top: true,
                    reverse: false,
                },
                PolicySetBindingOp {
                    source: PolicySetBindingSource::PolicySet(set_b),
                    top: true,
                    reverse: true,
                },
            ],
        );

        let resolved = ScriptExecutor::resolve_policy_set_group_ids(
            "test.policySet",
            set_a,
            &set_map,
            &set_group_map,
            &bindings,
            &mut Vec::new(),
        )
        .expect("policy set bindings should resolve");

        assert_eq!(resolved, vec![group_c, group_d, group_a, group_b]);
    }

    #[test]
    fn detects_policy_set_binding_cycle() {
        let set_a = set_id();
        let set_b = set_id();

        let mut set_map = HashMap::new();
        for (id, order_index, name) in [(set_a, 0, "A"), (set_b, 1, "B")] {
            set_map.insert(
                id,
                PolicySetTable {
                    id,
                    script_id: ScriptId::new_v7(),
                    order_index,
                    data: SqlJson(crate::domain::scripts::policy::PolicySetInfo {
                        name: name.to_string(),
                        note: String::new(),
                    }),
                },
            );
        }

        let set_group_map = HashMap::new();
        let mut bindings = HashMap::new();
        bindings.insert(
            set_a,
            vec![PolicySetBindingOp {
                source: PolicySetBindingSource::PolicySet(set_b),
                top: false,
                reverse: false,
            }],
        );
        bindings.insert(
            set_b,
            vec![PolicySetBindingOp {
                source: PolicySetBindingSource::PolicySet(set_a),
                top: false,
                reverse: false,
            }],
        );

        let error = ScriptExecutor::resolve_policy_set_group_ids(
            "test.policySet",
            set_a,
            &set_map,
            &set_group_map,
            &bindings,
            &mut Vec::new(),
        )
        .expect_err("cycle should be rejected");

        assert!(error.to_string().contains("策略集绑定存在循环"));
    }

    #[test]
    fn resolves_policy_group_bindings_with_top_and_reverse() {
        let group_a = group_id();
        let group_b = group_id();
        let policy_a = policy_id();
        let policy_b = policy_id();
        let policy_c = policy_id();
        let policy_d = policy_id();

        let mut group_map = HashMap::new();
        for (id, order_index, name) in [(group_a, 0, "A"), (group_b, 1, "B")] {
            group_map.insert(
                id,
                PolicyGroupTable {
                    id,
                    script_id: ScriptId::new_v7(),
                    order_index,
                    data: SqlJson(crate::domain::scripts::policy::PolicyGroupInfo {
                        name: name.to_string(),
                        note: String::new(),
                    }),
                },
            );
        }

        let mut group_policy_map = HashMap::new();
        group_policy_map.insert(group_a, vec![policy_a]);
        group_policy_map.insert(group_b, vec![policy_c, policy_d]);

        let mut bindings = HashMap::new();
        bindings.insert(
            group_a,
            vec![
                PolicyGroupBindingOp {
                    source: PolicyGroupBindingSource::Policy(policy_b),
                    top: false,
                    reverse: false,
                },
                PolicyGroupBindingOp {
                    source: PolicyGroupBindingSource::Policy(policy_c),
                    top: true,
                    reverse: true,
                },
                PolicyGroupBindingOp {
                    source: PolicyGroupBindingSource::PolicyGroup(group_b),
                    top: true,
                    reverse: true,
                },
            ],
        );

        let resolved = ScriptExecutor::resolve_policy_group_policy_ids(
            "test.policyGroup",
            group_a,
            &group_map,
            &group_policy_map,
            &bindings,
            &mut Vec::new(),
        )
        .expect("policy group bindings should resolve");

        assert_eq!(resolved, vec![policy_d, policy_c, policy_c, policy_a, policy_b]);
    }

    #[test]
    fn detects_policy_group_binding_cycle() {
        let group_a = group_id();
        let group_b = group_id();

        let mut group_map = HashMap::new();
        for (id, order_index, name) in [(group_a, 0, "A"), (group_b, 1, "B")] {
            group_map.insert(
                id,
                PolicyGroupTable {
                    id,
                    script_id: ScriptId::new_v7(),
                    order_index,
                    data: SqlJson(crate::domain::scripts::policy::PolicyGroupInfo {
                        name: name.to_string(),
                        note: String::new(),
                    }),
                },
            );
        }

        let group_policy_map = HashMap::new();
        let mut bindings = HashMap::new();
        bindings.insert(
            group_a,
            vec![PolicyGroupBindingOp {
                source: PolicyGroupBindingSource::PolicyGroup(group_b),
                top: false,
                reverse: false,
            }],
        );
        bindings.insert(
            group_b,
            vec![PolicyGroupBindingOp {
                source: PolicyGroupBindingSource::PolicyGroup(group_a),
                top: false,
                reverse: false,
            }],
        );

        let error = ScriptExecutor::resolve_policy_group_policy_ids(
            "test.policyGroup",
            group_a,
            &group_map,
            &group_policy_map,
            &bindings,
            &mut Vec::new(),
        )
        .expect_err("cycle should be rejected");

        assert!(error.to_string().contains("策略组绑定存在循环"));
    }
}
