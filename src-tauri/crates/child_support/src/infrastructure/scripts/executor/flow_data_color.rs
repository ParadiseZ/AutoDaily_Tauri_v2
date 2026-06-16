impl ScriptExecutor {
    async fn execute_color_compare_step(
        &mut self,
        input_var: &str,
        out_var: &str,
        target_text: Option<&str>,
        is_font: bool,
        target_color: &ColorRgb,
        method: &ColorCompareMethod,
        region: Option<&RegionRect>,
        then_steps: &[Step],
    ) -> ExecuteResult<ControlFlow> {
        if let Some(timeout_flow) = self
            .record_progress_evidence(
                "data.colorCompare",
                format!(
                    "ColorCompare 比较输入变量 {} 到输出变量 {}",
                    input_var, out_var
                ),
            )
            .await?
        {
            return Ok(timeout_flow);
        }
        let Some(input) = self.read_runtime_var(input_var).await else {
            self.set_runtime_var(out_var, Dynamic::from(Array::new()))
                .await?;
            return Ok(ControlFlow::Next);
        };

        let items = Self::deserialize_dynamic_value::<Vec<OcrResult>>(&input).map_err(|error| {
            Self::execute_error(
                "data.colorCompare",
                format!("输入变量[{}]不是兼容的 OCR 结果集: {}", input_var, error),
            )
        })?;
        let capture = self.require_last_capture_image("data.colorCompare").await?;
        let target_lab = Self::rgb_to_oklab(target_color);
        let candidates = Self::filter_ocr_items_for_color_compare(&items, target_text);
        let matched: Vec<OcrResult> = candidates
            .into_iter()
            .filter(|item| {
                region
                    .map(|region| Self::bounding_box_center_in_region(&item.bounding_box, region))
                    .unwrap_or(true)
            })
            .filter(|item| {
                Self::ocr_item_matches_color(
                    capture.as_ref(),
                    item,
                    is_font,
                    target_lab,
                    method,
                )
            })
            .cloned()
            .collect();

        let has_matched = !matched.is_empty();
        self.set_runtime_var(
            out_var,
            to_dynamic(matched).map_err(|error| {
                Self::execute_error(
                    "data.colorCompare",
                    format!("颜色比较结果写入变量失败: {}", error),
                )
            })?,
        )
        .await?;
        if has_matched && !then_steps.is_empty() {
            return self.execute(then_steps).await;
        }
        Ok(ControlFlow::Next)
    }

    async fn require_last_capture_image(&self, step_type: &str) -> ExecuteResult<Arc<RgbaImage>> {
        let ctx = self.runtime_ctx.read().await;
        ctx.observation
            .last_capture_image
            .clone()
            .ok_or_else(|| {
                Self::execute_error(
                    step_type,
                    "当前运行时没有可用截图，请先执行 Capture 或激活图像上下文".to_string(),
                )
            })
    }

    fn filter_ocr_items_for_color_compare<'a>(
        items: &'a [OcrResult],
        target_text: Option<&str>,
    ) -> Vec<&'a OcrResult> {
        let target_text = target_text
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let Some(target_text) = target_text else {
            return items.iter().collect();
        };

        let exact: Vec<_> = items.iter().filter(|item| item.txt.trim() == target_text).collect();
        if !exact.is_empty() {
            return exact;
        }

        items.iter()
            .filter(|item| item.txt.contains(target_text))
            .collect()
    }

    fn ocr_item_matches_color(
        image: &RgbaImage,
        item: &OcrResult,
        is_font: bool,
        target_lab: OklabColor,
        method: &ColorCompareMethod,
    ) -> bool {
        let Some(inner_rect) = Self::clamp_bbox_to_image(&item.bounding_box, image) else {
            return false;
        };
        let ring_rect = Self::expand_rect_for_ring(&inner_rect, image.width(), image.height());
        let threshold = Self::color_compare_threshold(method);

        if is_font {
            let candidate_colors =
                Self::extract_font_candidate_colors(image, inner_rect, ring_rect);
            return candidate_colors.into_iter().any(|candidate| {
                Self::oklab_distance(candidate, target_lab) <= threshold
            });
        }

        Self::extract_background_color(image, inner_rect, ring_rect)
            .map(|candidate| Self::oklab_distance(candidate, target_lab) <= threshold)
            .unwrap_or(false)
    }

    fn color_compare_threshold(method: &ColorCompareMethod) -> f32 {
        match method {
            ColorCompareMethod::OklabDistance { threshold } => threshold.max(0.0),
        }
    }

    fn extract_font_candidate_colors(
        image: &RgbaImage,
        inner_rect: BoundingBox,
        ring_rect: BoundingBox,
    ) -> Vec<OklabColor> {
        let inner_pixels = Self::collect_rect_pixels(image, &inner_rect);
        if inner_pixels.is_empty() {
            return Vec::new();
        }

        let background_color = Self::extract_background_color(image, inner_rect, ring_rect);
        let filtered_pixels = if let Some(background_color) = background_color {
            let separated: Vec<_> = inner_pixels
                .iter()
                .copied()
                .filter(|pixel| Self::oklab_distance(*pixel, background_color) >= 0.05)
                .collect();
            if separated.len() >= 12 {
                separated
            } else {
                inner_pixels.clone()
            }
        } else {
            inner_pixels.clone()
        };

        Self::cluster_colors(&filtered_pixels, 3)
            .into_iter()
            .take(3)
            .map(|cluster| cluster.center)
            .collect()
    }

    fn extract_background_color(
        image: &RgbaImage,
        inner_rect: BoundingBox,
        ring_rect: BoundingBox,
    ) -> Option<OklabColor> {
        let ring_pixels = Self::collect_ring_pixels(image, ring_rect, inner_rect);
        if ring_pixels.is_empty() {
            return None;
        }

        Self::cluster_colors(&ring_pixels, 3)
            .into_iter()
            .next()
            .map(|cluster| cluster.center)
    }

    fn clamp_bbox_to_image(bbox: &BoundingBox, image: &RgbaImage) -> Option<BoundingBox> {
        let max_x = image.width().saturating_sub(1) as i32;
        let max_y = image.height().saturating_sub(1) as i32;
        let x1 = bbox.x1.clamp(0, max_x);
        let y1 = bbox.y1.clamp(0, max_y);
        let x2 = bbox.x2.clamp(0, max_x);
        let y2 = bbox.y2.clamp(0, max_y);
        (x1 < x2 && y1 < y2).then_some(BoundingBox::new(x1, y1, x2, y2))
    }

    fn expand_rect_for_ring(
        rect: &BoundingBox,
        image_width: u32,
        image_height: u32,
    ) -> BoundingBox {
        let width = (rect.x2 - rect.x1).max(1);
        let height = (rect.y2 - rect.y1).max(1);
        let pad_x = ((width as f32 * 0.15).round() as i32).max(2);
        let pad_y = ((height as f32 * 0.15).round() as i32).max(2);
        BoundingBox::new(
            (rect.x1 - pad_x).clamp(0, image_width.saturating_sub(1) as i32),
            (rect.y1 - pad_y).clamp(0, image_height.saturating_sub(1) as i32),
            (rect.x2 + pad_x).clamp(0, image_width.saturating_sub(1) as i32),
            (rect.y2 + pad_y).clamp(0, image_height.saturating_sub(1) as i32),
        )
    }

    fn collect_rect_pixels(image: &RgbaImage, rect: &BoundingBox) -> Vec<OklabColor> {
        let mut pixels = Vec::new();
        for y in rect.y1..=rect.y2 {
            for x in rect.x1..=rect.x2 {
                let pixel = image.get_pixel(x as u32, y as u32);
                pixels.push(Self::rgb_to_oklab(&ColorRgb {
                    r: pixel[0],
                    g: pixel[1],
                    b: pixel[2],
                }));
            }
        }
        pixels
    }

    fn collect_ring_pixels(
        image: &RgbaImage,
        outer_rect: BoundingBox,
        inner_rect: BoundingBox,
    ) -> Vec<OklabColor> {
        let mut pixels = Vec::new();
        for y in outer_rect.y1..=outer_rect.y2 {
            for x in outer_rect.x1..=outer_rect.x2 {
                if x >= inner_rect.x1
                    && x <= inner_rect.x2
                    && y >= inner_rect.y1
                    && y <= inner_rect.y2
                {
                    continue;
                }

                let pixel = image.get_pixel(x as u32, y as u32);
                pixels.push(Self::rgb_to_oklab(&ColorRgb {
                    r: pixel[0],
                    g: pixel[1],
                    b: pixel[2],
                }));
            }
        }
        pixels
    }

    fn cluster_colors(points: &[OklabColor], limit: usize) -> Vec<ColorCluster> {
        if points.is_empty() || limit == 0 {
            return Vec::new();
        }

        let k = limit.min(points.len());
        let mut centers = Vec::with_capacity(k);
        centers.push(points[0]);
        while centers.len() < k {
            let next = points
                .iter()
                .copied()
                .max_by(|left, right| {
                    let left_distance = centers
                        .iter()
                        .map(|center| Self::oklab_distance(*left, *center))
                        .fold(f32::MAX, f32::min);
                    let right_distance = centers
                        .iter()
                        .map(|center| Self::oklab_distance(*right, *center))
                        .fold(f32::MAX, f32::min);
                    left_distance.total_cmp(&right_distance)
                })
                .unwrap_or(points[0]);
            if centers.iter().any(|center| Self::oklab_distance(*center, next) < 0.001) {
                break;
            }
            centers.push(next);
        }

        for _ in 0..8 {
            let mut groups: Vec<Vec<OklabColor>> = vec![Vec::new(); centers.len()];
            for point in points {
                let closest = centers
                    .iter()
                    .enumerate()
                    .min_by(|(_, left), (_, right)| {
                        Self::oklab_distance(*point, **left)
                            .total_cmp(&Self::oklab_distance(*point, **right))
                    })
                    .map(|(index, _)| index)
                    .unwrap_or(0);
                groups[closest].push(*point);
            }

            let mut moved = false;
            for (index, group) in groups.into_iter().enumerate() {
                if group.is_empty() {
                    continue;
                }
                let next_center = Self::mean_color(&group);
                if Self::oklab_distance(centers[index], next_center) > 0.001 {
                    centers[index] = next_center;
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }

        let mut groups: Vec<Vec<OklabColor>> = vec![Vec::new(); centers.len()];
        for point in points {
            let closest = centers
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    Self::oklab_distance(*point, **left)
                        .total_cmp(&Self::oklab_distance(*point, **right))
                })
                .map(|(index, _)| index)
                .unwrap_or(0);
            groups[closest].push(*point);
        }

        let mut clusters = Vec::new();
        for (center, members) in centers.into_iter().zip(groups.into_iter()) {
            if members.is_empty() {
                continue;
            }
            let mean_distance =
                members.iter().map(|member| Self::oklab_distance(*member, center)).sum::<f32>()
                    / members.len() as f32;
            clusters.push(ColorCluster {
                center,
                count: members.len(),
                mean_distance,
            });
        }

        clusters.sort_by(|left, right| {
            right
                .count
                .cmp(&left.count)
                .then_with(|| left.mean_distance.total_cmp(&right.mean_distance))
        });
        clusters
    }

    fn mean_color(points: &[OklabColor]) -> OklabColor {
        let mut sum = OklabColor::default();
        for point in points {
            sum.l += point.l;
            sum.a += point.a;
            sum.b += point.b;
        }
        let count = points.len() as f32;
        OklabColor {
            l: sum.l / count,
            a: sum.a / count,
            b: sum.b / count,
        }
    }

    fn oklab_distance(left: OklabColor, right: OklabColor) -> f32 {
        let dl = left.l - right.l;
        let da = left.a - right.a;
        let db = left.b - right.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    fn rgb_to_oklab(color: &ColorRgb) -> OklabColor {
        let srgb = |value: u8| value as f32 / 255.0;
        let to_linear = |value: f32| {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };

        let r = to_linear(srgb(color.r));
        let g = to_linear(srgb(color.g));
        let b = to_linear(srgb(color.b));

        let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
        let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
        let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;

        let l_cbrt = l.cbrt();
        let m_cbrt = m.cbrt();
        let s_cbrt = s.cbrt();

        OklabColor {
            l: 0.210_454_26 * l_cbrt + 0.793_617_8 * m_cbrt - 0.004_072_047 * s_cbrt,
            a: 1.977_998_5 * l_cbrt - 2.428_592_2 * m_cbrt + 0.450_593_7 * s_cbrt,
            b: 0.025_904_037 * l_cbrt + 0.782_771_77 * m_cbrt - 0.808_675_77 * s_cbrt,
        }
    }
}
