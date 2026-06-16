impl ScriptExecutor {
    fn select_ocr_result<'a>(
        items: &'a [OcrResult],
        target_text: Option<&str>,
    ) -> Option<&'a OcrResult> {
        Self::select_ocr_result_at(items, target_text, None)
    }

    fn select_ocr_result_at<'a>(
        items: &'a [OcrResult],
        target_text: Option<&str>,
        position: Option<u16>,
    ) -> Option<&'a OcrResult> {
        Self::select_positioned_match(Self::select_ocr_results(items, target_text), position)
    }

    fn select_ocr_results<'a>(
        items: &'a [OcrResult],
        target_text: Option<&str>,
    ) -> Vec<&'a OcrResult> {
        let target_text = target_text.map(str::trim).filter(|value| !value.is_empty());
        match target_text {
            Some(target) => {
                let exact: Vec<&OcrResult> = items
                    .iter()
                    .filter(|item| item.txt.trim() == target)
                    .collect();
                if exact.is_empty() {
                    items.iter().filter(|item| item.txt.contains(target)).collect()
                } else {
                    exact
                }
            }
            None => items.iter().collect(),
        }
    }

    fn select_det_result(items: &[DetResult], target_idx: Option<u32>) -> Option<&DetResult> {
        Self::select_det_result_at(items, target_idx, None)
    }

    fn select_det_result_at(
        items: &[DetResult],
        target_idx: Option<u32>,
        position: Option<u16>,
    ) -> Option<&DetResult> {
        Self::select_positioned_match(Self::select_det_results(items, target_idx), position)
    }

    fn select_det_results(items: &[DetResult], target_idx: Option<u32>) -> Vec<&DetResult> {
        match target_idx {
            Some(target) => items
                .iter()
                .filter(|item| item.index == target as i32)
                .collect(),
            None => items.iter().collect(),
        }
    }

    fn select_positioned_match<T>(matches: Vec<&T>, position: Option<u16>) -> Option<&T> {
        if matches.is_empty() {
            return None;
        }
        let index = match position {
            Some(999) => matches.len() - 1,
            Some(value) => usize::from(value).min(matches.len() - 1),
            None => 0,
        };
        matches.get(index).copied()
    }

    fn resolve_optional_text(
        &mut self,
        value: Option<&str>,
        expr: Option<&str>,
        step_type: &str,
    ) -> ExecuteResult<Option<String>> {
        if let Some(expr) = expr.map(str::trim).filter(|value| !value.is_empty()) {
            let dynamic = self.eval_dynamic(expr, step_type)?;
            return Self::dynamic_to_string(&dynamic)
                .map(|value| Some(value.trim().to_string()).filter(|value| !value.is_empty()))
                .ok_or_else(|| {
                    Self::execute_error(
                        step_type,
                        format!("文本表达式结果无法转为字符串: {}", expr),
                    )
                });
        }

        Ok(value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string))
    }
}
