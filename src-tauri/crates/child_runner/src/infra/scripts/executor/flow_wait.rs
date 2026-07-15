use regex::Regex;
use std::sync::OnceLock;

impl ScriptExecutor {
    async fn resolve_wait_duration_ms(
        &self,
        configured_ms: u64,
        input_var: Option<&str>,
        runtime_var: Option<&str>,
    ) -> u64 {
        let fallback_ms = Self::fallback_wait_ms(configured_ms);

        if let Some(runtime_var) = runtime_var.filter(|value| !value.trim().is_empty()) {
            match self.read_runtime_var(runtime_var).await {
                Some(value) => match Self::extract_wait_ms_from_runtime_ocr(&value) {
                    Ok(Some(parsed)) => return parsed,
                    Ok(None) => Log::warn(&format!(
                        "[ runtime ] WaitMs 运行时变量[{}]未匹配到 OCR 时间文本，回退到 {}ms",
                        runtime_var, fallback_ms
                    )),
                    Err(error) => Log::warn(&format!(
                        "[ runtime ] WaitMs 解析运行时变量[{}]失败：{}，回退到 {}ms",
                        runtime_var, error, fallback_ms
                    )),
                },
                None => Log::warn(&format!(
                    "[ runtime ] WaitMs 运行时变量[{}]不存在，回退到 {}ms",
                    runtime_var, fallback_ms
                )),
            }
        }

        if let Some(input_var) = input_var.filter(|value| !value.trim().is_empty()) {
            match self.read_runtime_var(input_var).await {
                Some(value) => match Self::parse_wait_ms_from_input(&value) {
                    Some(parsed) => return parsed,
                    None => Log::warn(&format!(
                        "[ runtime ] WaitMs 输入变量[{}]不是有效毫秒值，回退到 {}ms",
                        input_var, fallback_ms
                    )),
                },
                None => Log::warn(&format!(
                    "[ runtime ] WaitMs 输入变量[{}]不存在，回退到 {}ms",
                    input_var, fallback_ms
                )),
            }
        }

        fallback_ms
    }

    fn fallback_wait_ms(configured_ms: u64) -> u64 {
        if configured_ms == 0 { 1_000 } else { configured_ms }
    }

    fn parse_wait_ms_from_input(value: &Dynamic) -> Option<u64> {
        let parsed = Self::dynamic_to_number(value)?;
        if parsed.is_finite() && parsed >= 0.0 {
            Some(parsed.round() as u64)
        } else {
            None
        }
    }

    fn extract_wait_ms_from_runtime_ocr(value: &Dynamic) -> Result<Option<u64>, String> {
        if let Ok(items) = Self::deserialize_dynamic_value::<Vec<OcrResult>>(value) {
            for item in items {
                if let Some(wait_ms) = Self::extract_wait_ms_from_text(&item.txt) {
                    return Ok(Some(wait_ms));
                }
            }
            return Ok(None);
        }

        let json_value = from_dynamic::<Value>(value)
            .map_err(|error| format!("运行时变量无法转换为 JSON: {}", error))?;
        Ok(Self::extract_wait_ms_from_json_value(&json_value))
    }

    fn extract_wait_ms_from_json_value(value: &Value) -> Option<u64> {
        match value {
            Value::String(text) => Self::extract_wait_ms_from_text(text),
            Value::Array(items) => items
                .iter()
                .find_map(Self::extract_wait_ms_from_json_value),
            Value::Object(map) => {
                if let Some(text) = map.get("txt").and_then(Value::as_str) {
                    if let Some(wait_ms) = Self::extract_wait_ms_from_text(text) {
                        return Some(wait_ms);
                    }
                }
                if let Some(text) = map.get("text").and_then(Value::as_str) {
                    if let Some(wait_ms) = Self::extract_wait_ms_from_text(text) {
                        return Some(wait_ms);
                    }
                }
                map.values().find_map(Self::extract_wait_ms_from_json_value)
            }
            _ => None,
        }
    }

    fn extract_wait_ms_from_text(text: &str) -> Option<u64> {
        let normalized = text.replace('：', ":");
        for captures in Self::wait_time_regex().captures_iter(&normalized) {
            let matched = captures.get(0)?;
            let prev_is_digit = normalized[..matched.start()]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_digit());
            let next_is_digit = normalized[matched.end()..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_digit());
            if prev_is_digit || next_is_digit {
                continue;
            }

            let trailing_seconds = captures.get(3)?.as_str().parse::<u64>().ok()?;
            let middle = captures.get(2)?.as_str().parse::<u64>().ok()?;
            if let Some(hours) = captures.get(1) {
                let hours = hours.as_str().parse::<u64>().ok()?;
                if middle >= 60 || trailing_seconds >= 60 {
                    continue;
                }
                return Some(((hours * 60 * 60) + (middle * 60) + trailing_seconds) * 1_000);
            }

            return Some(((middle * 60) + trailing_seconds) * 1_000);
        }

        None
    }

    fn wait_time_regex() -> &'static Regex {
        static WAIT_TIME_REGEX: OnceLock<Regex> = OnceLock::new();
        WAIT_TIME_REGEX.get_or_init(|| {
            Regex::new(r"(?:(\d{1,2}):)?(\d{1,3}):(\d{2})")
                .expect("wait time regex should be valid")
        })
    }
}
