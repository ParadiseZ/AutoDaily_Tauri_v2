impl ScriptExecutor {
    fn point_to_absolute(point: &PointU16) -> Point<u16> {
        Point::new(point.x, point.y)
    }

    fn bounding_box_center_to_point(
        step_type: &str,
        target_label: &str,
        bounding_box: &BoundingBox,
    ) -> ExecuteResult<Point<u16>> {
        let center = bounding_box.center();
        let x = u16::try_from(center.x).map_err(|_| {
            Self::execute_error(
                step_type,
                format!("{}中心点 x 坐标越界: {}", target_label, center.x),
            )
        })?;
        let y = u16::try_from(center.y).map_err(|_| {
            Self::execute_error(
                step_type,
                format!("{}中心点 y 坐标越界: {}", target_label, center.y),
            )
        })?;
        Ok(Point::new(x, y))
    }

    async fn apply_click_random_offset(&self, point: Point<u16>) -> ExecuteResult<Point<u16>> {
        let offset = {
            let ctx = self.runtime_ctx.read().await;
            ctx.execution
                .script_info
                .as_ref()
                .map(|script| script.runtime_settings.click_random_offset)
                .unwrap_or(0)
        };
        if offset == 0 {
            return Ok(point);
        }

        let screen_size = self.ensure_screen_size().await?;
        let span = i32::from(offset);
        let dx = Self::random_i32_inclusive(-span, span);
        let dy = Self::random_i32_inclusive(-span, span);
        let max_x = screen_size.0.saturating_sub(1) as i32;
        let max_y = screen_size.1.saturating_sub(1) as i32;
        let x = (i32::from(point.x) + dx).clamp(0, max_x) as u16;
        let y = (i32::from(point.y) + dy).clamp(0, max_y) as u16;
        Ok(Point::new(x, y))
    }

    async fn apply_click_fixed_offset(
        &self,
        point: Point<u16>,
        offset_x: i32,
        offset_y: i32,
    ) -> ExecuteResult<Point<u16>> {
        if offset_x == 0 && offset_y == 0 {
            return Ok(point);
        }

        let screen_size = self.ensure_screen_size().await?;
        let max_x = screen_size.0.saturating_sub(1) as i32;
        let max_y = screen_size.1.saturating_sub(1) as i32;
        let x = (i32::from(point.x) + offset_x).clamp(0, max_x) as u16;
        let y = (i32::from(point.y) + offset_y).clamp(0, max_y) as u16;
        Ok(Point::new(x, y))
    }

    fn random_i32_inclusive(min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos() as u64)
            .unwrap_or(0);
        let mut hasher = XxHash3_64::default();
        hasher.write_u64(nanos);
        let range = (i64::from(max) - i64::from(min) + 1) as u64;
        min + (hasher.finish() % range) as i32
    }

    fn action_requires_wait(action: &Action) -> bool {
        matches!(
            action,
            Action::Click { .. }
                | Action::Swipe { .. }
                | Action::LongClick { .. }
                | Action::Reboot
                | Action::Back
                | Action::Home
                | Action::InputText { .. }
                | Action::LaunchApp { .. }
                | Action::StopApp { .. }
        )
    }

    fn percent_point_to_absolute(
        point: &PointF32,
        screen_size: (u32, u32),
    ) -> ExecuteResult<Point<u16>> {
        let (width, height) = screen_size;
        if width == 0 || height == 0 {
            return Err(Self::execute_error(
                "action.percentPoint",
                "屏幕尺寸无效，无法换算百分比坐标".to_string(),
            ));
        }
        let max_x = width.saturating_sub(1) as f32;
        let max_y = height.saturating_sub(1) as f32;
        let x = (point.x.clamp(0.0, 1.0) * max_x).round() as u16;
        let y = (point.y.clamp(0.0, 1.0) * max_y).round() as u16;
        Ok(Point::new(x, y))
    }
}
