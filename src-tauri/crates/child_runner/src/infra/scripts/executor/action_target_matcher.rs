impl ScriptExecutor {
    fn resolve_u16_point(
        &mut self,
        value: &PointU16,
        expr: Option<&str>,
        step_type: &str,
    ) -> ExecuteResult<PointU16> {
        let Some(expr) = expr.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(value.clone());
        };

        let dynamic = self.eval_dynamic(expr, step_type)?;
        let (x, y) = Self::dynamic_to_point_components(&dynamic, step_type, expr)?;
        if x < 0.0 || y < 0.0 {
            return Err(Self::execute_error(
                step_type,
                format!("点位变量结果不能为负数: {}", expr),
            ));
        }
        if x > u16::MAX as f64 || y > u16::MAX as f64 {
            return Err(Self::execute_error(
                step_type,
                format!("点位变量结果超出坐标范围: {}", expr),
            ));
        }

        Ok(PointU16 {
            x: x.round() as u16,
            y: y.round() as u16,
        })
    }

    fn resolve_f32_point(
        &mut self,
        value: &PointF32,
        expr: Option<&str>,
        step_type: &str,
    ) -> ExecuteResult<PointF32> {
        let Some(expr) = expr.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(value.clone());
        };

        let dynamic = self.eval_dynamic(expr, step_type)?;
        let (x, y) = Self::dynamic_to_point_components(&dynamic, step_type, expr)?;
        Ok(PointF32 {
            x: x as f32,
            y: y as f32,
        })
    }

    fn dynamic_to_point_components(
        value: &Dynamic,
        step_type: &str,
        expr: &str,
    ) -> ExecuteResult<(f64, f64)> {
        let json_value = rhai::serde::from_dynamic::<serde_json::Value>(value).map_err(|error| {
            Self::execute_error(
                step_type,
                format!("点位变量结果无法转为 JSON 对象: {} ({})", expr, error),
            )
        })?;

        let serde_json::Value::Object(map) = json_value else {
            return Err(Self::execute_error(
                step_type,
                format!("点位变量结果必须是包含 x / y 的 JSON 对象: {}", expr),
            ));
        };

        let parse_component = |key: &str| {
            map.get(key).and_then(|entry| match entry {
                serde_json::Value::Number(number) => number.as_f64(),
                serde_json::Value::String(text) => text.trim().parse::<f64>().ok(),
                _ => None,
            })
        };

        let Some(x) = parse_component("x") else {
            return Err(Self::execute_error(
                step_type,
                format!("点位变量结果缺少数字字段 x: {}", expr),
            ));
        };
        let Some(y) = parse_component("y") else {
            return Err(Self::execute_error(
                step_type,
                format!("点位变量结果缺少数字字段 y: {}", expr),
            ));
        };

        if !x.is_finite() || !y.is_finite() {
            return Err(Self::execute_error(
                step_type,
                format!("点位变量结果必须是有限数字: {}", expr),
            ));
        }

        Ok((x, y))
    }

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

    fn resolve_optional_u32(
        &mut self,
        value: Option<u32>,
        expr: Option<&str>,
        step_type: &str,
    ) -> ExecuteResult<Option<u32>> {
        if let Some(expr) = expr.map(str::trim).filter(|value| !value.is_empty()) {
            let dynamic = self.eval_dynamic(expr, step_type)?;
            if let Some(number) = Self::dynamic_to_number(&dynamic) {
                if number < 0.0 {
                    return Err(Self::execute_error(
                        step_type,
                        format!("整数表达式结果不能为负数: {}", expr),
                    ));
                }
                return Ok(Some(number.trunc() as u32));
            }
            return Err(Self::execute_error(
                step_type,
                format!("整数表达式结果无法转为数字: {}", expr),
            ));
        }

        Ok(value)
    }
}
