struct ActionTraceBuilder;

impl ActionTraceBuilder {
    fn build_simple_action_trace(kind: PolicyActionKind) -> PolicyActionTrace {
        Self::build_action_trace(kind, PolicyActionSource::Custom, Vec::new())
    }

    fn build_action_trace(
        kind: PolicyActionKind,
        source: PolicyActionSource,
        targets: Vec<PolicyActionTarget>,
    ) -> PolicyActionTrace {
        let signature = Self::build_action_signature(&kind, &source, &targets);
        PolicyActionTrace {
            action_index: 0,
            kind,
            source,
            signature,
            targets,
        }
    }

    fn build_action_signature(
        kind: &PolicyActionKind,
        source: &PolicyActionSource,
        targets: &[PolicyActionTarget],
    ) -> String {
        let mut hasher = XxHash3_64::default();
        hasher.write(b"policy-action:v1");
        hasher.write(&[Self::action_kind_code(kind)]);
        hasher.write(&[Self::action_source_code(source)]);
        for target in targets {
            hasher.write(&[Self::target_role_code(&target.role)]);
            let (x, y) = target
                .point
                .as_ref()
                .map(|point| (point.x, point.y))
                .unwrap_or_default();
            hasher.write(&x.to_le_bytes());
            hasher.write(&y.to_le_bytes());
            if let Some(box_area) = target.box_area.as_ref() {
                hasher.write(&box_area.x1.to_le_bytes());
                hasher.write(&box_area.y1.to_le_bytes());
                hasher.write(&box_area.x2.to_le_bytes());
                hasher.write(&box_area.y2.to_le_bytes());
            } else {
                hasher.write(&0i32.to_le_bytes());
                hasher.write(&0i32.to_le_bytes());
                hasher.write(&0i32.to_le_bytes());
                hasher.write(&0i32.to_le_bytes());
            }
            ScriptExecutor::write_hash_segment(
                &mut hasher,
                target.text.as_deref().unwrap_or("").as_bytes(),
            );
            hasher.write(&target.label_id.unwrap_or_default().to_le_bytes());
        }
        format!("action:v1:{:016x}", hasher.finish())
    }

    fn build_point_target(role: PolicyActionTargetRole, point: Point<u16>) -> PolicyActionTarget {
        PolicyActionTarget {
            role,
            point: Some(PointU16 { x: point.x, y: point.y }),
            box_area: None,
            text: None,
            label_id: None,
        }
    }

    fn build_ocr_target(
        role: PolicyActionTargetRole,
        point: Point<u16>,
        item: &OcrResult,
    ) -> PolicyActionTarget {
        PolicyActionTarget {
            role,
            point: Some(PointU16 { x: point.x, y: point.y }),
            box_area: Some(item.bounding_box.clone()),
            text: Some(item.txt.clone()),
            label_id: None,
        }
    }

    fn build_det_target(
        role: PolicyActionTargetRole,
        point: Point<u16>,
        item: &DetResult,
    ) -> PolicyActionTarget {
        PolicyActionTarget {
            role,
            point: Some(PointU16 { x: point.x, y: point.y }),
            box_area: Some(item.bounding_box.clone()),
            text: Some(item.label.clone()),
            label_id: Some(item.index),
        }
    }

    fn action_kind_code(kind: &PolicyActionKind) -> u8 {
        match kind {
            PolicyActionKind::Unknown => 0,
            PolicyActionKind::Click => 1,
            PolicyActionKind::Swipe => 2,
            PolicyActionKind::Input => 3,
            PolicyActionKind::Press => 4,
            PolicyActionKind::Reboot => 5,
            PolicyActionKind::StartApp => 6,
            PolicyActionKind::StopApp => 7,
            PolicyActionKind::Back => 8,
            PolicyActionKind::Home => 9,
            PolicyActionKind::Menu => 10,
            PolicyActionKind::None => 11,
        }
    }

    fn action_source_code(source: &PolicyActionSource) -> u8 {
        match source {
            PolicyActionSource::Ocr => 1,
            PolicyActionSource::Det => 2,
            PolicyActionSource::Label => 3,
            PolicyActionSource::Fixed => 4,
            PolicyActionSource::Text => 5,
            PolicyActionSource::Custom => 6,
        }
    }

    fn target_role_code(role: &PolicyActionTargetRole) -> u8 {
        match role {
            PolicyActionTargetRole::Primary => 1,
            PolicyActionTargetRole::Secondary => 2,
            PolicyActionTargetRole::Start => 3,
            PolicyActionTargetRole::End => 4,
            PolicyActionTargetRole::Path => 5,
        }
    }
}
