// storage/mod.rs

use std::sync::RwLock;

#[cfg(feature = "surrealdb")]
use surrealdb::{
    engine::any::{self, Any},
    opt::auth::Root,
    Surreal,
};

use crate::{config, utils::error::Error};

#[cfg(feature = "postgres")]
pub type AsyncPgPool = DeadpoolPool<AsyncPgConnection>;

#[cfg(feature = "surrealdb")]
pub type SurrealDbPool = Surreal<Any>;

#[cfg(feature = "postgres")]
lazy_static! {
    static ref ASYNC_PG_POOL: RwLock<Option<AsyncPgPool>> = RwLock::new(None);
}

#[cfg(feature = "surrealdb")]
lazy_static! {
    static ref ASYNC_SL_POOL: RwLock<Option<SurrealDbPool>> = RwLock::new(None);
}

#[cfg(feature = "postgres")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/postgres");

pub async fn setup() -> Result<(), Error> {
    let conf = config::get();

    tracing::info!("🦋 Initializing the database connection pool...");

    #[cfg(feature = "postgres")]
    {
        let pool = AsyncPgPool::new(&conf.database_url).await?;

        *ASYNC_PG_POOL.write().unwrap() = Some(pool);
    }

    #[cfg(feature = "surrealdb")]
    {
        let db = any::connect(&conf.surrealdb.dsn).await?;

        db.signin(Root {
            username: &conf.surrealdb.username,
            password: &conf.surrealdb.password,
        })
        .await?;

        *ASYNC_SL_POOL.write().unwrap() = Some(db);
    }

    tracing::info!("📦 Running the database migrations...");

    Ok(())
}
