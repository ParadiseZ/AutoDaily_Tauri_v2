use sqlx::SqlitePool;
use tokio::sync::OnceCell;

mod bootstrap;
mod migrations;
mod repo;
mod schema;

pub(crate) static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

pub use bootstrap::{get_pool, init_db, init_db_with_path};
pub use repo::DbRepo;
