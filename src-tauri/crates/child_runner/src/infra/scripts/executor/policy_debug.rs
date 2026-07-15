impl ScriptExecutor {
    async fn capture_policy_debug_observation(
        &mut self,
        debug_target_label: &str,
    ) -> ExecuteResult<()> {
        Log::info(&format!(
            "[ policy_debug ] 开始{}调试截图与视觉分析",
            debug_target_label
        ));
        let image = Arc::new(self.capture_device_screenshot("debug.policy").await?);
        self.activate_image_context("debug.policy", image, Some("runtime.policyDebugCapture"))
            .await
    }

    async fn log_policy_debug_observation(
        &self,
        debug_target_label: &str,
    ) -> ExecuteResult<()> {
        let det_results = self
            .read_runtime_result_vec::<DetResult>("runtime.detResults", "debug.policy", "检测")
            .await?;
        let ocr_results = self
            .read_runtime_result_vec::<OcrResult>("runtime.ocrResults", "debug.policy", "OCR")
            .await?;

        Log::info(&format!(
            "[ policy_debug ] {}截图完成: det={} ocr={}",
            debug_target_label,
            det_results.len(),
            ocr_results.len()
        ));

        for (index, item) in det_results.iter().take(10).enumerate() {
            let center = item.bounding_box.center();
            Log::info(&format!(
                "[ policy_debug ] DET[{}] label={} idx={} score={:.3} center=({}, {})",
                index, item.label, item.index, item.score, center.x, center.y
            ));
        }
        if det_results.len() > 10 {
            Log::info(&format!(
                "[ policy_debug ] DET 结果已截断展示，其余 {} 条省略",
                det_results.len() - 10
            ));
        }

        for (index, item) in ocr_results.iter().take(10).enumerate() {
            let center = item.bounding_box.center();
            Log::info(&format!(
                "[ policy_debug ] OCR[{}] text=\"{}\" center=({}, {})",
                index, item.txt, center.x, center.y
            ));
        }
        if ocr_results.len() > 10 {
            Log::info(&format!(
                "[ policy_debug ] OCR 结果已截断展示，其余 {} 条省略",
                ocr_results.len() - 10
            ));
        }

        Ok(())
    }

    fn log_policy_debug_result(&self, result: &PolicyExecutionResult) {
        Log::info(&format!(
            "[ policy_debug ] 最终结果: matched={} policySet={:?} policyGroup={:?} policy={:?}",
            result.matched, result.policy_set_id, result.policy_group_id, result.policy_id
        ));

        for (round_index, round) in result.rounds.iter().enumerate() {
            Log::info(&format!(
                "[ policy_debug ] round[{}]: matched={} set={:?} group={:?} policy={:?} pageFingerprints={} actionSignatures={}",
                round_index,
                round.matched,
                round.policy_set_id,
                round.policy_group_id,
                round.policy_id,
                round.page_fingerprints.len(),
                round.action_signatures.len()
            ));
            if !round.page_fingerprints.is_empty() {
                Log::info(&format!(
                    "[ policy_debug ] round[{}] pageFingerprints={}",
                    round_index,
                    round.page_fingerprints.join(" -> ")
                ));
            }
            for action in &round.actions {
                Log::info(&format!(
                    "[ policy_debug ] round[{}] action[{}]: kind={:?} source={:?} signature={} targets={}",
                    round_index,
                    action.action_index,
                    action.kind,
                    action.source,
                    action.signature,
                    Self::format_policy_action_targets(&action.targets)
                ));
            }
        }
    }

    fn format_policy_action_targets(targets: &[PolicyActionTarget]) -> String {
        if targets.is_empty() {
            return "[]".to_string();
        }

        targets
            .iter()
            .map(|target| {
                let point = target
                    .point
                    .as_ref()
                    .map(|point| format!("point=({}, {})", point.x, point.y))
                    .unwrap_or_else(|| "point=<none>".to_string());
                let text = target
                    .text
                    .as_ref()
                    .map(|text| format!("text=\"{}\"", text))
                    .unwrap_or_else(|| "text=<none>".to_string());
                let label = target
                    .label_id
                    .map(|label_id| format!("label={}", label_id))
                    .unwrap_or_else(|| "label=<none>".to_string());
                format!("{:?}({}; {}; {})", target.role, point, text, label)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    async fn begin_active_policy_round_trace(
        &mut self,
        policy_id: PolicyId,
        policy_name: String,
        base_click_pos: u16,
    ) {
        self.active_policy_round = Some(ActivePolicyRoundTrace::default());
        self.active_policy_context = Some(ActivePolicyContext {
            policy_id,
            policy_name,
            base_click_pos,
        });
        if let Some(fingerprint) = self.current_page_fingerprint().await {
            self.push_active_policy_page_fingerprint(fingerprint);
        }
    }

    fn take_active_policy_round_trace(&mut self) -> ActivePolicyRoundTrace {
        self.active_policy_context = None;
        self.active_policy_round.take().unwrap_or_default()
    }
}
