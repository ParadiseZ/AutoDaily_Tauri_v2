impl ScriptExecutor {
    fn register_rhai_step_helpers(&mut self) {
        self.engine
            .register_fn("point", |x: INT, y: INT| -> Result<Dynamic, Box<EvalAltResult>> {
                Ok(ScriptExecutor::to_rhai_dynamic(
                    "point",
                    &RegionPoint::Point {
                        p: PointU16 {
                            x: ScriptExecutor::int_to_u16("point", "x", x)?,
                            y: ScriptExecutor::int_to_u16("point", "y", y)?,
                        },
                    },
                )?)
            });
        self.engine.register_fn(
            "percent",
            |x: FLOAT, y: FLOAT| -> Result<Dynamic, Box<EvalAltResult>> {
                Ok(ScriptExecutor::to_rhai_dynamic(
                    "percent",
                    &RegionPoint::Percent {
                        p: PointF32 {
                            x: x as f32,
                            y: y as f32,
                        },
                    },
                )?)
            },
        );
        self.engine.register_fn(
            "rgb",
            |r: INT, g: INT, b: INT| -> Result<Dynamic, Box<EvalAltResult>> {
                Ok(ScriptExecutor::to_rhai_dynamic(
                    "rgb",
                    &ColorRgb {
                        r: ScriptExecutor::int_to_u8("rgb", "r", r)?,
                        g: ScriptExecutor::int_to_u8("rgb", "g", g)?,
                        b: ScriptExecutor::int_to_u8("rgb", "b", b)?,
                    },
                )?)
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "click",
            move |target: Dynamic| -> Result<(), Box<EvalAltResult>> {
                let mode = ScriptExecutor::click_mode_from_target("click", &target)?;
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "click",
                    ScriptExecutor::build_action_step(Action::Click {
                        offset_x: 0,
                        offset_y: 0,
                        mode,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "click_text",
            move |input_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "click_text",
                    ScriptExecutor::build_action_step(Action::Click {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::Txt {
                            input_var,
                            txt: None,
                            txt_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "click_text",
            move |input_var: String, text: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "click_text",
                    ScriptExecutor::build_action_step(Action::Click {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::Txt {
                            input_var,
                            txt: Some(text),
                            txt_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "click_label",
            move |input_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "click_label",
                    ScriptExecutor::build_action_step(Action::Click {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::LabelIdx {
                            input_var,
                            idx: None,
                            idx_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "click_label",
            move |input_var: String, idx: INT| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "click_label",
                    ScriptExecutor::build_action_step(Action::Click {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::LabelIdx {
                            input_var,
                            idx: Some(ScriptExecutor::int_to_u32("click_label", "idx", idx)?),
                            idx_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "long_click",
            move |target: Dynamic| -> Result<(), Box<EvalAltResult>> {
                let mode = ScriptExecutor::click_mode_from_target("long_click", &target)?;
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "long_click",
                    ScriptExecutor::build_action_step(Action::LongClick {
                        offset_x: 0,
                        offset_y: 0,
                        mode,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "long_click_text",
            move |input_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "long_click_text",
                    ScriptExecutor::build_action_step(Action::LongClick {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::Txt {
                            input_var,
                            txt: None,
                            txt_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "long_click_text",
            move |input_var: String, text: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "long_click_text",
                    ScriptExecutor::build_action_step(Action::LongClick {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::Txt {
                            input_var,
                            txt: Some(text),
                            txt_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "long_click_label",
            move |input_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "long_click_label",
                    ScriptExecutor::build_action_step(Action::LongClick {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::LabelIdx {
                            input_var,
                            idx: None,
                            idx_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "long_click_label",
            move |input_var: String, idx: INT| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "long_click_label",
                    ScriptExecutor::build_action_step(Action::LongClick {
                        offset_x: 0,
                        offset_y: 0,
                        mode: ClickMode::LabelIdx {
                            input_var,
                            idx: Some(ScriptExecutor::int_to_u32(
                                "long_click_label",
                                "idx",
                                idx,
                            )?),
                            idx_expr: None,
                            enable_filter: true,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "swipe",
            move |from: Dynamic, to: Dynamic| -> Result<(), Box<EvalAltResult>> {
                let mode = ScriptExecutor::swipe_mode_from_targets("swipe", &from, &to)?;
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "swipe",
                    ScriptExecutor::build_action_step(Action::Swipe {
                        duration: 300,
                        mode,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "swipe",
            move |from: Dynamic, to: Dynamic, duration_ms: INT| -> Result<(), Box<EvalAltResult>> {
                let mode = ScriptExecutor::swipe_mode_from_targets("swipe", &from, &to)?;
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "swipe",
                    ScriptExecutor::build_action_step(Action::Swipe {
                        duration: ScriptExecutor::int_to_u64("swipe", "duration_ms", duration_ms)?,
                        mode,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "swipe_text",
            move |input_var: String,
                  from_text: String,
                  to_text: String|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "swipe_text",
                    ScriptExecutor::build_action_step(Action::Swipe {
                        duration: 300,
                        mode: SwipeMode::Txt {
                            input_var,
                            from: Some(from_text),
                            to: Some(to_text),
                            from_expr: None,
                            to_expr: None,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "swipe_text",
            move |input_var: String,
                  from_text: String,
                  to_text: String,
                  duration_ms: INT|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "swipe_text",
                    ScriptExecutor::build_action_step(Action::Swipe {
                        duration: ScriptExecutor::int_to_u64(
                            "swipe_text",
                            "duration_ms",
                            duration_ms,
                        )?,
                        mode: SwipeMode::Txt {
                            input_var,
                            from: Some(from_text),
                            to: Some(to_text),
                            from_expr: None,
                            to_expr: None,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "swipe_label",
            move |input_var: String, from_idx: INT, to_idx: INT| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "swipe_label",
                    ScriptExecutor::build_action_step(Action::Swipe {
                        duration: 300,
                        mode: SwipeMode::LabelIdx {
                            input_var,
                            from: ScriptExecutor::int_to_u16("swipe_label", "from_idx", from_idx)?,
                            to: ScriptExecutor::int_to_u16("swipe_label", "to_idx", to_idx)?,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "swipe_label",
            move |input_var: String,
                  from_idx: INT,
                  to_idx: INT,
                  duration_ms: INT|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "swipe_label",
                    ScriptExecutor::build_action_step(Action::Swipe {
                        duration: ScriptExecutor::int_to_u64(
                            "swipe_label",
                            "duration_ms",
                            duration_ms,
                        )?,
                        mode: SwipeMode::LabelIdx {
                            input_var,
                            from: ScriptExecutor::int_to_u16("swipe_label", "from_idx", from_idx)?,
                            to: ScriptExecutor::int_to_u16("swipe_label", "to_idx", to_idx)?,
                        },
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "capture",
            move |out_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "capture",
                    ScriptExecutor::build_action_step(Action::Capture { output_var: out_var }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "input_text",
            move |text: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "input_text",
                    ScriptExecutor::build_action_step(Action::InputText { text }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine
            .register_fn("back", move || -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "back",
                    ScriptExecutor::build_action_step(Action::Back),
                )
            });

        let queue = self.rhai_step_queue.clone();
        self.engine
            .register_fn("home", move || -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "home",
                    ScriptExecutor::build_action_step(Action::Home),
                )
            });

        let queue = self.rhai_step_queue.clone();
        self.engine
            .register_fn("reboot", move || -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "reboot",
                    ScriptExecutor::build_action_step(Action::Reboot),
                )
            });

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "launch_app",
            move |pkg_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "launch_app",
                    ScriptExecutor::build_action_step(Action::LaunchApp {
                        pkg_name,
                        pkg_name_expr: None,
                        activity_name: String::new(),
                        activity_name_expr: None,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "launch_app",
            move |pkg_name: String, activity_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "launch_app",
                    ScriptExecutor::build_action_step(Action::LaunchApp {
                        pkg_name,
                        pkg_name_expr: None,
                        activity_name,
                        activity_name_expr: None,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "stop_app",
            move |pkg_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "stop_app",
                    ScriptExecutor::build_action_step(Action::StopApp {
                        pkg_name,
                        pkg_name_expr: None,
                    }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "detect",
            move |input_var: String, out_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "detect",
                    ScriptExecutor::build_vision_step(VisionNode::Detect { input_var, out_var }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "ocr",
            move |input_var: String, out_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "ocr",
                    ScriptExecutor::build_vision_step(VisionNode::Ocr { input_var, out_var }),
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine
            .register_fn("wait_ms", move |ms: INT| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "wait_ms",
                    ScriptExecutor::build_flow_step(FlowControl::WaitMs {
                        ms: ScriptExecutor::int_to_u64("wait_ms", "ms", ms)?,
                        input_var: None,
                        runtime_var: None,
                    }),
                )
            });

        let queue = self.rhai_step_queue.clone();
        self.engine
            .register_fn("stop_script", move || -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_step(
                    &queue,
                    "stop_script",
                    ScriptExecutor::build_flow_step(FlowControl::StopScript),
                )
            });

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "link_task",
            move |task_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "link_task",
                    QueuedRhaiOp::LinkTaskByName { task_name },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "set_task_enabled",
            move |task_name: String, value: bool| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "set_task_enabled",
                    QueuedRhaiOp::SetTaskStateByName {
                        task_name,
                        status: StateStatus::Enabled { value },
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "set_task_skip",
            move |task_name: String, value: bool| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "set_task_skip",
                    QueuedRhaiOp::SetTaskStateByName {
                        task_name,
                        status: StateStatus::Skip { value },
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "set_task_done",
            move |task_name: String, value: bool| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "set_task_done",
                    QueuedRhaiOp::SetTaskStateByName {
                        task_name,
                        status: StateStatus::Done { value },
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "set_policy_skip",
            move |policy_name: String, value: bool| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "set_policy_skip",
                    QueuedRhaiOp::SetPolicyStateByName {
                        policy_name,
                        status: StateStatus::Skip { value },
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "set_policy_done",
            move |policy_name: String, value: bool| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "set_policy_done",
                    QueuedRhaiOp::SetPolicyStateByName {
                        policy_name,
                        status: StateStatus::Done { value },
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "add_policies",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "add_policies",
                    QueuedRhaiOp::AddPoliciesByName {
                        source_name,
                        target_name,
                        top: false,
                        reverse: false,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "add_policies",
            move |source_name: String,
                  target_name: String,
                  top: bool,
                  reverse: bool|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "add_policies",
                    QueuedRhaiOp::AddPoliciesByName {
                        source_name,
                        target_name,
                        top,
                        reverse,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "remove_policies",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "remove_policies",
                    QueuedRhaiOp::RemovePoliciesByName {
                        source_name,
                        target_name,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "bind_policy_group",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "bind_policy_group",
                    QueuedRhaiOp::BindPolicyGroupByName {
                        source_name,
                        target_name,
                        top: false,
                        reverse: false,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "bind_policy_group",
            move |source_name: String,
                  target_name: String,
                  top: bool,
                  reverse: bool|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "bind_policy_group",
                    QueuedRhaiOp::BindPolicyGroupByName {
                        source_name,
                        target_name,
                        top,
                        reverse,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "remove_policy_group",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "remove_policy_group",
                    QueuedRhaiOp::RemovePolicyGroupByName {
                        source_name,
                        target_name,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "add_policy_groups",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "add_policy_groups",
                    QueuedRhaiOp::AddPolicyGroupsByName {
                        source_name,
                        target_name,
                        top: false,
                        reverse: false,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "add_policy_groups",
            move |source_name: String,
                  target_name: String,
                  top: bool,
                  reverse: bool|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "add_policy_groups",
                    QueuedRhaiOp::AddPolicyGroupsByName {
                        source_name,
                        target_name,
                        top,
                        reverse,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "unload_policy_group",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "unload_policy_group",
                    QueuedRhaiOp::UnloadPolicyGroupByName {
                        source_name,
                        target_name,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "bind_policy",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "bind_policy",
                    QueuedRhaiOp::BindPolicyByName {
                        source_name,
                        target_name,
                        top: false,
                        reverse: false,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "bind_policy",
            move |source_name: String,
                  target_name: String,
                  top: bool,
                  reverse: bool|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "bind_policy",
                    QueuedRhaiOp::BindPolicyByName {
                        source_name,
                        target_name,
                        top,
                        reverse,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "unload_policy",
            move |source_name: String, target_name: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "unload_policy",
                    QueuedRhaiOp::UnloadPolicyByName {
                        source_name,
                        target_name,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "handle_policy_set",
            move |target_names: Array, out_var: String| -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "handle_policy_set",
                    QueuedRhaiOp::HandlePolicySetByName {
                        target_names: ScriptExecutor::deserialize_rhai_helper(
                            "handle_policy_set",
                            &Dynamic::from_array(target_names),
                        )?,
                        det_input_var: "runtime.detResults".to_string(),
                        ocr_input_var: "runtime.ocrResults".to_string(),
                        search_hits_var: "runtime.searchHits".to_string(),
                        out_var,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "handle_policy_set",
            move |target_names: Array,
                  det_input_var: String,
                  ocr_input_var: String,
                  search_hits_var: String,
                  out_var: String|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "handle_policy_set",
                    QueuedRhaiOp::HandlePolicySetByName {
                        target_names: ScriptExecutor::deserialize_rhai_helper(
                            "handle_policy_set",
                            &Dynamic::from_array(target_names),
                        )?,
                        det_input_var,
                        ocr_input_var,
                        search_hits_var,
                        out_var,
                    },
                )
            },
        );

        let queue = self.rhai_step_queue.clone();
        self.engine.register_fn(
            "handle_policy",
            move |target_names: Array,
                  input_var: String,
                  out_var: String|
                  -> Result<(), Box<EvalAltResult>> {
                ScriptExecutor::enqueue_rhai_op(
                    &queue,
                    "handle_policy",
                    QueuedRhaiOp::HandlePolicyByName {
                        target_names: ScriptExecutor::deserialize_rhai_helper(
                            "handle_policy",
                            &Dynamic::from_array(target_names),
                        )?,
                        input_var,
                        out_var,
                    },
                )
            },
        );
    }

    fn rhai_helper_error(message: String) -> Box<EvalAltResult> {
        Box::new(EvalAltResult::ErrorRuntime(
            Dynamic::from(message),
            rhai::Position::NONE,
        ))
    }

    fn to_rhai_dynamic<T>(helper_name: &'static str, value: &T) -> Result<Dynamic, Box<EvalAltResult>>
    where
        T: Serialize,
    {
        to_dynamic(value).map_err(|error| {
            Self::rhai_helper_error(format!(
                "{}() 返回值生成失败: {}",
                helper_name, error
            ))
        })
    }

    fn deserialize_rhai_helper<T>(
        helper_name: &'static str,
        config: &Dynamic,
    ) -> Result<T, Box<EvalAltResult>>
    where
        T: DeserializeOwned,
    {
        from_dynamic(config).map_err(|error| {
            Self::rhai_helper_error(format!("{}() 参数解析失败: {}", helper_name, error))
        })
    }

    fn int_to_u8(
        helper_name: &'static str,
        field_name: &'static str,
        value: INT,
    ) -> Result<u8, Box<EvalAltResult>> {
        value.try_into().map_err(|_| {
            Self::rhai_helper_error(format!(
                "{}() 的 {} 超出 u8 范围: {}",
                helper_name, field_name, value
            ))
        })
    }

    fn int_to_u16(
        helper_name: &'static str,
        field_name: &'static str,
        value: INT,
    ) -> Result<u16, Box<EvalAltResult>> {
        value.try_into().map_err(|_| {
            Self::rhai_helper_error(format!(
                "{}() 的 {} 超出 u16 范围: {}",
                helper_name, field_name, value
            ))
        })
    }

    fn int_to_u32(
        helper_name: &'static str,
        field_name: &'static str,
        value: INT,
    ) -> Result<u32, Box<EvalAltResult>> {
        value.try_into().map_err(|_| {
            Self::rhai_helper_error(format!(
                "{}() 的 {} 超出 u32 范围: {}",
                helper_name, field_name, value
            ))
        })
    }

    fn int_to_u64(
        helper_name: &'static str,
        field_name: &'static str,
        value: INT,
    ) -> Result<u64, Box<EvalAltResult>> {
        value.try_into().map_err(|_| {
            Self::rhai_helper_error(format!(
                "{}() 的 {} 超出 u64 范围: {}",
                helper_name, field_name, value
            ))
        })
    }

    fn click_mode_from_target(
        helper_name: &'static str,
        target: &Dynamic,
    ) -> Result<ClickMode, Box<EvalAltResult>> {
        match Self::deserialize_rhai_helper::<RegionPoint>(helper_name, target)? {
            RegionPoint::Point { p } => Ok(ClickMode::Point { p, p_expr: None }),
            RegionPoint::Percent { p } => Ok(ClickMode::Percent { p, p_expr: None }),
        }
    }

    fn swipe_mode_from_targets(
        helper_name: &'static str,
        from: &Dynamic,
        to: &Dynamic,
    ) -> Result<SwipeMode, Box<EvalAltResult>> {
        let from = Self::deserialize_rhai_helper::<RegionPoint>(helper_name, from)?;
        let to = Self::deserialize_rhai_helper::<RegionPoint>(helper_name, to)?;
        match (from, to) {
            (RegionPoint::Point { p: from }, RegionPoint::Point { p: to }) => {
                Ok(SwipeMode::Point { from, to })
            }
            (RegionPoint::Percent { p: from }, RegionPoint::Percent { p: to }) => {
                Ok(SwipeMode::Percent { from, to })
            }
            _ => Err(Self::rhai_helper_error(format!(
                "{}() 的 from/to 必须同为 point() 或同为 percent()",
                helper_name
            ))),
        }
    }

    fn build_action_step(action: Action) -> Step {
        Step {
            id: None,
            source_id: None,
            target_id: None,
            label: None,
            skip_flag: false,
            kind: StepKind::Action { exec_max: 0, a: action },
        }
    }

    fn build_flow_step(flow: FlowControl) -> Step {
        Step {
            id: None,
            source_id: None,
            target_id: None,
            label: None,
            skip_flag: false,
            kind: StepKind::FlowControl { a: flow },
        }
    }

    fn build_task_control_step(task: TaskControl) -> Step {
        Step {
            id: None,
            source_id: None,
            target_id: None,
            label: None,
            skip_flag: false,
            kind: StepKind::TaskControl { a: task },
        }
    }

    fn build_vision_step(vision: VisionNode) -> Step {
        Step {
            id: None,
            source_id: None,
            target_id: None,
            label: None,
            skip_flag: false,
            kind: StepKind::Vision { a: vision },
        }
    }

    async fn load_task_bundle(&self, step_type: &str) -> ExecuteResult<Vec<ScriptTaskTable>> {
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
        Self::parse_bundle_json(step_type, "tasks_json", &snapshot.tasks_json)
    }

    fn resolve_named_id<T: Copy>(
        step_type: &str,
        helper_name: &'static str,
        field_name: &'static str,
        raw_name: &str,
        items: &[(T, String)],
    ) -> ExecuteResult<T> {
        if let Some((id, _)) = items.iter().find(|(_, name)| name == raw_name) {
            return Ok(*id);
        }

        Err(Self::execute_error(
            step_type,
            format!(
                "{}() 的 {} [{}] 不存在",
                helper_name, field_name, raw_name
            ),
        ))
    }

    async fn resolve_task_id_by_name(
        &self,
        helper_name: &'static str,
        task_name: &str,
    ) -> ExecuteResult<TaskId> {
        let tasks = self.load_task_bundle(helper_name).await?;
        let items = tasks
            .into_iter()
            .map(|task| (task.id, task.name))
            .collect::<Vec<_>>();
        Self::resolve_named_id(helper_name, helper_name, "task_name", task_name, &items)
    }

    async fn resolve_policy_id_by_name(
        &self,
        helper_name: &'static str,
        policy_name: &str,
    ) -> ExecuteResult<PolicyId> {
        let bundle = self.load_policy_bundle(helper_name).await?;
        let items = bundle
            .policies
            .into_iter()
            .map(|policy| (policy.id, policy.data.0.name))
            .collect::<Vec<_>>();
        Self::resolve_named_id(helper_name, helper_name, "policy_name", policy_name, &items)
    }

    async fn resolve_policy_group_id_by_name(
        &self,
        helper_name: &'static str,
        group_name: &str,
    ) -> ExecuteResult<PolicyGroupId> {
        let bundle = self.load_policy_bundle(helper_name).await?;
        let items = bundle
            .policy_groups
            .into_iter()
            .map(|group| (group.id, group.data.0.name))
            .collect::<Vec<_>>();
        Self::resolve_named_id(helper_name, helper_name, "group_name", group_name, &items)
    }

    async fn resolve_policy_set_id_by_name(
        &self,
        helper_name: &'static str,
        set_name: &str,
    ) -> ExecuteResult<PolicySetId> {
        let bundle = self.load_policy_bundle(helper_name).await?;
        let items = bundle
            .policy_sets
            .into_iter()
            .map(|set| (set.id, set.data.0.name))
            .collect::<Vec<_>>();
        Self::resolve_named_id(helper_name, helper_name, "set_name", set_name, &items)
    }

    async fn materialize_queued_rhai_op(
        &self,
        helper_name: &'static str,
        op: QueuedRhaiOp,
    ) -> ExecuteResult<Step> {
        match op {
            QueuedRhaiOp::Step(step) => Ok(step),
            QueuedRhaiOp::LinkTaskByName { task_name } => Ok(Self::build_flow_step(
                FlowControl::Link {
                    target: self.resolve_task_id_by_name(helper_name, &task_name).await?,
                },
            )),
            QueuedRhaiOp::SetTaskStateByName { task_name, status } => Ok(
                Self::build_task_control_step(TaskControl::SetState {
                    target: StateTarget::Task {
                        id: self.resolve_task_id_by_name(helper_name, &task_name).await?,
                    },
                    targets: Vec::new(),
                    status,
                }),
            ),
            QueuedRhaiOp::SetPolicyStateByName {
                policy_name,
                status,
            } => Ok(Self::build_task_control_step(TaskControl::SetState {
                target: StateTarget::Policy {
                    id: self
                        .resolve_policy_id_by_name(helper_name, &policy_name)
                        .await?,
                },
                targets: Vec::new(),
                status,
            })),
            QueuedRhaiOp::AddPoliciesByName {
                source_name,
                target_name,
                top,
                reverse,
            } => Ok(Self::build_flow_step(FlowControl::AddPolicies {
                source: self
                    .resolve_policy_set_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_set_id_by_name(helper_name, &target_name)
                    .await?,
                top,
                reverse,
            })),
            QueuedRhaiOp::RemovePoliciesByName {
                source_name,
                target_name,
            } => Ok(Self::build_flow_step(FlowControl::RemovePolicies {
                source: self
                    .resolve_policy_set_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_set_id_by_name(helper_name, &target_name)
                    .await?,
            })),
            QueuedRhaiOp::BindPolicyGroupByName {
                source_name,
                target_name,
                top,
                reverse,
            } => Ok(Self::build_flow_step(FlowControl::BindPolicyGroup {
                source: self
                    .resolve_policy_group_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_set_id_by_name(helper_name, &target_name)
                    .await?,
                top,
                reverse,
            })),
            QueuedRhaiOp::RemovePolicyGroupByName {
                source_name,
                target_name,
            } => Ok(Self::build_flow_step(FlowControl::RemovePolicyGroup {
                source: self
                    .resolve_policy_group_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_set_id_by_name(helper_name, &target_name)
                    .await?,
            })),
            QueuedRhaiOp::AddPolicyGroupsByName {
                source_name,
                target_name,
                top,
                reverse,
            } => Ok(Self::build_flow_step(FlowControl::AddPolicyGroups {
                source: self
                    .resolve_policy_group_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_group_id_by_name(helper_name, &target_name)
                    .await?,
                top,
                reverse,
            })),
            QueuedRhaiOp::UnloadPolicyGroupByName {
                source_name,
                target_name,
            } => Ok(Self::build_flow_step(FlowControl::UnloadPolicyGroup {
                source: self
                    .resolve_policy_group_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_group_id_by_name(helper_name, &target_name)
                    .await?,
            })),
            QueuedRhaiOp::BindPolicyByName {
                source_name,
                target_name,
                top,
                reverse,
            } => Ok(Self::build_flow_step(FlowControl::BindPolicy {
                source: self
                    .resolve_policy_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_group_id_by_name(helper_name, &target_name)
                    .await?,
                top,
                reverse,
            })),
            QueuedRhaiOp::UnloadPolicyByName {
                source_name,
                target_name,
            } => Ok(Self::build_flow_step(FlowControl::UnloadPolicy {
                source: self
                    .resolve_policy_id_by_name(helper_name, &source_name)
                    .await?,
                target: self
                    .resolve_policy_group_id_by_name(helper_name, &target_name)
                    .await?,
            })),
            QueuedRhaiOp::HandlePolicySetByName {
                target_names,
                det_input_var,
                ocr_input_var,
                search_hits_var,
                out_var,
            } => {
                let mut target = Vec::with_capacity(target_names.len());
                for name in target_names {
                    target.push(self.resolve_policy_set_id_by_name(helper_name, &name).await?);
                }
                Ok(Self::build_flow_step(FlowControl::HandlePolicySet {
                    target,
                    det_input_var,
                    ocr_input_var,
                    search_hits_var,
                    out_var,
                }))
            }
            QueuedRhaiOp::HandlePolicyByName {
                target_names,
                input_var,
                out_var,
            } => {
                let mut target = Vec::with_capacity(target_names.len());
                for name in target_names {
                    target.push(self.resolve_policy_id_by_name(helper_name, &name).await?);
                }
                Ok(Self::build_flow_step(FlowControl::HandlePolicy {
                    target,
                    input_var,
                    out_var,
                }))
            }
        }
    }

    fn push_rhai_step_queue(&self) {
        let mut guard = self
            .rhai_step_queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.push(Vec::new());
    }

    fn pop_rhai_step_queue(&self) -> Vec<QueuedRhaiStep> {
        let mut guard = self
            .rhai_step_queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.pop().unwrap_or_default()
    }

    fn enqueue_rhai_step(
        queue: &Arc<StdMutex<Vec<Vec<QueuedRhaiStep>>>>,
        helper_name: &'static str,
        step: Step,
    ) -> Result<(), Box<EvalAltResult>> {
        Self::enqueue_rhai_op(queue, helper_name, QueuedRhaiOp::Step(step))
    }

    fn enqueue_rhai_op(
        queue: &Arc<StdMutex<Vec<Vec<QueuedRhaiStep>>>>,
        helper_name: &'static str,
        op: QueuedRhaiOp,
    ) -> Result<(), Box<EvalAltResult>> {
        let mut guard = queue.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(current) = guard.last_mut() else {
            return Err(Self::rhai_helper_error(format!(
                "{}() 只能在 Rhai 代码步骤里调用",
                helper_name
            )));
        };
        current.push(QueuedRhaiStep { helper_name, op });
        Ok(())
    }

    async fn flush_rhai_step_queue(&mut self) -> ExecuteResult<ControlFlow> {
        for queued in self.pop_rhai_step_queue() {
            let step = self
                .materialize_queued_rhai_op(queued.helper_name, queued.op)
                .await?;
            Log::debug(&format!(
                "[ executor ] rhai helper -> helper={}, kind={}",
                queued.helper_name,
                Self::describe_step_kind(&step)
            ));
            let flow = self.execute_step(&step).await?;
            if !matches!(flow, ControlFlow::Next) {
                return Ok(flow);
            }
        }
        Ok(ControlFlow::Next)
    }

    pub(crate) async fn execute_rhai_block(
        &mut self,
        code: &str,
        step_type: &str,
    ) -> ExecuteResult<(Dynamic, ControlFlow)> {
        self.push_rhai_step_queue();

        let result = match self.eval_rhai_block(code, step_type) {
            Ok(value) => value,
            Err(error) => {
                self.pop_rhai_step_queue();
                return Err(error);
            }
        };

        self.sync_scope_root_to_runtime_var_map("input").await;
        self.sync_scope_root_to_runtime_var_map("runtime").await;

        let flow = match self.flush_rhai_step_queue().await {
            Ok(flow) => flow,
            Err(error) => return Err(error),
        };

        self.sync_scope_root_to_runtime_var_map("input").await;
        self.sync_scope_root_to_runtime_var_map("runtime").await;

        Ok((result, flow))
    }
}
