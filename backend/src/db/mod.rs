pub mod compat;
pub mod models;

#[cfg(feature = "postgres")]
mod pg;
#[cfg(feature = "desktop")]
mod sqlite;

#[cfg(feature = "postgres")]
pub use pg::*;
#[cfg(feature = "desktop")]
pub use sqlite::*;

#[cfg(feature = "postgres")]
pub type DbPool = sqlx::PgPool;
#[cfg(feature = "desktop")]
pub type DbPool = sqlx::SqlitePool;
