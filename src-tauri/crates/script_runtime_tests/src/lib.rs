pub mod config;
mod image_source;
pub mod runner;

#[cfg(test)]
mod tests {
    use ad_kernel::ids::{ScriptId, TaskId};
    use child_runner::testkit::{TestTaskRunRequest, run_task_test};
    use domain_script::{
        Action, ScriptInfo, ScriptProfile, ScriptTask, ScriptTaskProfile, Step, StepKind,
        TaskCycle, TaskRowType, TaskTone, TaskTriggerMode,
    };
    use runner_protocol::message::ScriptBundleSnapshot;
    use serde_json::{Value, json};

    #[tokio::test]
    async fn complete_bundle_task_runs_through_the_real_scheduler() {
        let script_id = ScriptId::new_v7();
        let task_id = TaskId::new_v7();
        let mut script_info = ScriptInfo::default();
        script_info.name = "runtime-test".to_string();
        let script = ScriptProfile {
            id: script_id,
            info: script_info,
        };
        let now = chrono::Utc::now();
        let task = ScriptTaskProfile {
            id: task_id,
            script_id,
            name: "记录返回操作".to_string(),
            description: String::new(),
            row_type: TaskRowType::Task,
            trigger_mode: TaskTriggerMode::RootOnly,
            record_schedule: false,
            section_id: None,
            indent_level: 0,
            default_task_cycle: TaskCycle::EveryRun,
            exec_max: 0,
            show_enabled_toggle: true,
            default_enabled: true,
            task_tone: TaskTone::Normal,
            is_hidden: false,
            task: ScriptTask {
                ui_data: Value::Null,
                variables: Value::Null,
                steps: vec![Step {
                    id: None,
                    source_id: None,
                    target_id: None,
                    label: Some("返回".to_string()),
                    skip_flag: false,
                    kind: StepKind::Action {
                        exec_max: 0,
                        a: Action::Back,
                    },
                }],
            },
            created_at: now,
            updated_at: now,
            deleted_at: None,
            is_deleted: false,
            index: 0,
        };
        let bundle = ScriptBundleSnapshot {
            script_id,
            script_json: serde_json::to_string(&script).unwrap(),
            tasks_json: serde_json::to_string(&vec![task]).unwrap(),
            policies_json: "[]".to_string(),
            policy_groups_json: "[]".to_string(),
            policy_sets_json: "[]".to_string(),
            group_policies_json: "[]".to_string(),
            set_groups_json: "[]".to_string(),
        };

        let result = run_task_test(TestTaskRunRequest {
            bundle,
            task_id,
            screenshots: Vec::new(),
            capture_vision_frames: Vec::new(),
            detect_vision_frames: Vec::new(),
            ocr_vision_frames: Vec::new(),
            use_real_vision: false,
            template_values_json: None,
        })
        .await
        .unwrap();

        assert_eq!(result["execution"]["outcome"], json!("completed"));
        assert_eq!(result["operations"], json!([{ "type": "back" }]));
        assert_eq!(result["taskStates"][task_id.to_string()]["done"], true);
    }
}
