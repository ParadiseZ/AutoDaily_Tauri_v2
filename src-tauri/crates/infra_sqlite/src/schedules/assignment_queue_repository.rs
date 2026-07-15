use crate::bootstrap::get_pool;
use ad_kernel::ids::{AssignmentId, DeviceId};

const ASSIGNMENT_TABLE: &str = "device_script_assignments";

pub async fn compact_assignment_indices(device_id: DeviceId) -> Result<Vec<AssignmentId>, String> {
    let rows = sqlx::query_as::<_, (String, i64)>(&format!(
        "SELECT id, `index` FROM {ASSIGNMENT_TABLE} WHERE device_id = ? ORDER BY `index` ASC, id ASC"
    ))
    .bind(device_id.to_string())
    .fetch_all(get_pool())
    .await
    .map_err(|error| error.to_string())?;

    let mut ordered_ids = Vec::with_capacity(rows.len());
    for (index, (id, old_index)) in rows.into_iter().enumerate() {
        let assignment_id =
            AssignmentId::from(uuid::Uuid::parse_str(&id).map_err(|error| error.to_string())?);
        ordered_ids.push(assignment_id);
        let index = u32::try_from(index).map_err(|_| "队列项数量超过排序范围".to_string())?;
        if old_index != i64::from(index) {
            update_index(assignment_id, index).await?;
        }
    }
    Ok(ordered_ids)
}

pub async fn reorder_assignment_indices(assignment_ids: &[AssignmentId]) -> Result<(), String> {
    for (index, assignment_id) in assignment_ids.iter().copied().enumerate() {
        let index = u32::try_from(index).map_err(|_| "队列项数量超过排序范围".to_string())?;
        update_index(assignment_id, index).await?;
    }
    Ok(())
}

async fn update_index(assignment_id: AssignmentId, index: u32) -> Result<(), String> {
    sqlx::query(&format!(
        "UPDATE {ASSIGNMENT_TABLE} SET `index` = ? WHERE id = ? AND `index` <> ?"
    ))
    .bind(index)
    .bind(assignment_id.to_string())
    .bind(index)
    .execute(get_pool())
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}
