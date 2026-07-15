use ad_kernel::ids::DeviceId;
use domain_device::{DeviceConfig, DeviceProfile};
use sqlx::FromRow;
use sqlx::types::Json;
use uuid::Uuid;

#[derive(FromRow)]
struct DeviceRow {
    id: String,
    data: Json<DeviceConfig>,
}

impl TryFrom<DeviceRow> for DeviceProfile {
    type Error = String;

    fn try_from(row: DeviceRow) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(&row.id).map_err(|error| error.to_string())?;
        Ok(Self {
            id: DeviceId::from(id),
            config: row.data.0,
        })
    }
}

pub async fn get_device(device_id: DeviceId) -> Result<Option<DeviceProfile>, String> {
    sqlx::query_as::<_, DeviceRow>("SELECT id, `data` FROM devices WHERE id = ?")
        .bind(device_id.to_string())
        .fetch_optional(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .map(TryInto::try_into)
        .transpose()
}

pub async fn get_all_devices() -> Result<Vec<DeviceProfile>, String> {
    sqlx::query_as::<_, DeviceRow>("SELECT id, `data` FROM devices ORDER BY id")
        .fetch_all(crate::bootstrap::get_pool())
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

pub async fn save_device(profile: &DeviceProfile) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO devices (id, `data`) VALUES (?, ?)
         ON CONFLICT(id) DO UPDATE SET `data` = excluded.`data`",
    )
    .bind(profile.id.to_string())
    .bind(Json(&profile.config))
    .execute(crate::bootstrap::get_pool())
    .await
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub async fn delete_device_with_assignments(device_id: DeviceId) -> Result<(), String> {
    let mut tx = crate::bootstrap::get_pool()
        .begin()
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM device_script_assignments WHERE device_id = ?")
        .bind(device_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    sqlx::query("DELETE FROM devices WHERE id = ?")
        .bind(device_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    tx.commit().await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_a_device_row_without_exposing_sqlx_json() {
        let profile = DeviceProfile::try_from(DeviceRow {
            id: "018f0f61-8c6f-7b26-9f24-5fc3cf249109".to_string(),
            data: Json(DeviceConfig::default()),
        })
        .unwrap();

        assert!(profile.config.supports_window_capture());
        assert_eq!(
            profile.id.to_string(),
            "018f0f61-8c6f-7b26-9f24-5fc3cf249109"
        );
    }
}
