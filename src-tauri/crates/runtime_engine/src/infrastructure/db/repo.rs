use super::bootstrap::get_pool;
use serde::Serialize;
use sqlx::types::Json;
use sqlx::FromRow;

/// 数据库操作仓库
pub struct DbRepo;

impl DbRepo {
    /// 这里的泛型 T 是你的 Data 部分 (例如 DeviceConfig)
    /// 返回 (ID, Data) 的元组列表
    pub async fn get_all<T>(table: &str) -> Result<Vec<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin + Send + Sync,
    {
        let pool = get_pool();
        let query = format!("SELECT id, `data` FROM {}", table);
        let rows: Vec<T> = sqlx::query_as(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    /// 根据 ID 获取单个记录
    pub async fn get_by_id<T>(table: &str, id: &str) -> Result<Option<T>, String>
    where
        T: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin + Send + Sync,
    {
        let pool = get_pool();
        let query = format!("SELECT id,`data` FROM {} WHERE id = ?", table);
        let row: Option<T> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(row)
    }

    /// 插入或更新 ID + Data 模式的数据
    /// 你不需要手动转换 JSON，sqlx 会处理
    pub async fn upsert_id_data<T>(table: &str, id: &str, data: &Json<T>) -> Result<(), String>
    where
        T: Serialize + Send + Sync,
    {
        let pool = get_pool();
        let query = format!(
            "INSERT INTO {} (id, `data`) VALUES (?, ?)
             ON CONFLICT(id) DO UPDATE SET `data` = excluded.`data`",
            table
        );

        sqlx::query(&query)
            .bind(id)
            .bind(data)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 删除数据
    pub async fn delete(table: &str, id: &str) -> Result<(), String> {
        let pool = get_pool();
        let query = format!("DELETE FROM {} WHERE id = ?", table);
        sqlx::query(&query)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
