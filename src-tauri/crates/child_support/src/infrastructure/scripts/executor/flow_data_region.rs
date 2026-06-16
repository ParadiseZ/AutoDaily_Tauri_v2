impl ScriptExecutor {
    async fn resolve_region_rect(
        &self,
        top_left: &RegionPoint,
        bottom_right: &RegionPoint,
    ) -> ExecuteResult<Option<RegionRect>> {
        if Self::region_point_is_default(top_left) && Self::region_point_is_default(bottom_right) {
            return Ok(None);
        }

        let screen_size = self.ensure_screen_size().await?;
        let start = Self::region_point_to_absolute(top_left, screen_size)?;
        let end = Self::region_point_to_absolute(bottom_right, screen_size)?;
        let x1 = start.x.min(end.x);
        let y1 = start.y.min(end.y);
        let x2 = start.x.max(end.x);
        let y2 = start.y.max(end.y);
        Ok(Some(RegionRect { x1, y1, x2, y2 }))
    }

    fn region_point_is_default(point: &RegionPoint) -> bool {
        match point {
            RegionPoint::Point { p } => p.x == 0 && p.y == 0,
            RegionPoint::Percent { p } => p.x == 0.0 && p.y == 0.0,
        }
    }

    fn region_point_to_absolute(
        point: &RegionPoint,
        screen_size: (u32, u32),
    ) -> ExecuteResult<Point<i32>> {
        match point {
            RegionPoint::Point { p } => Ok(Point::new(i32::from(p.x), i32::from(p.y))),
            RegionPoint::Percent { p } => {
                let absolute = Self::percent_point_to_absolute(p, screen_size)?;
                Ok(Point::new(i32::from(absolute.x), i32::from(absolute.y)))
            }
        }
    }

    fn dynamic_item_in_region(item: &Dynamic, region: &RegionRect) -> ExecuteResult<bool> {
        let value = from_dynamic::<Value>(item).map_err(|error| {
            Self::execute_error(
                "data.filter.region",
                format!("Filter 区域筛选读取条目失败: {}", error),
            )
        })?;
        let Some(bounding_box) = value
            .get("bounding_box")
            .or_else(|| value.get("boundingBox"))
            .cloned()
        else {
            return Ok(false);
        };
        let bounding_box = serde_json::from_value::<BoundingBox>(bounding_box).map_err(|error| {
            Self::execute_error(
                "data.filter.region",
                format!("Filter 区域筛选解析 bounding_box 失败: {}", error),
            )
        })?;
        Ok(Self::bounding_box_center_in_region(&bounding_box, region))
    }

    fn bounding_box_center_in_region(bounding_box: &BoundingBox, region: &RegionRect) -> bool {
        let center = bounding_box.center();
        center.x >= region.x1
            && center.x <= region.x2
            && center.y >= region.y1
            && center.y <= region.y2
    }
}
