use secrecy::{ExposeSecret, SecretString};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{MySqlPool, PgPool, SqlitePool};
use std::time::Duration;
use tracing::info;

#[allow(dead_code)]
pub enum DbBackend {
    Pg(PgPool),
    Mysql(MySqlPool),
    Sqlite(SqlitePool),
    Disconnected,
}

#[allow(dead_code)]
pub enum EngineType {
    Postgres,
    Mysql,
    Sqlite,
}

impl DbBackend {
    pub async fn connect(dsn: &SecretString, engine_type: EngineType) -> Result<Self, String> {
        let dsn_str = dsn.expose_secret();

        match engine_type {
            EngineType::Postgres => {
                match PgPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(Duration::from_secs(10))
                    .connect(dsn_str)
                    .await
                {
                    Ok(pool) => match pool.acquire().await {
                        Ok(_conn) => {
                            info!("Connected to PostgreSQL");
                            Ok(Self::Pg(pool))
                        },
                        Err(e) => Err(format!("Pool acquire failed: {e}")),
                    },
                    Err(e) => Err(format!("{e}")),
                }
            },
            EngineType::Mysql => {
                match MySqlPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(Duration::from_secs(10))
                    .connect(dsn_str)
                    .await
                {
                    Ok(pool) => match pool.acquire().await {
                        Ok(_conn) => {
                            info!("Connected to MySQL");
                            Ok(Self::Mysql(pool))
                        },
                        Err(e) => Err(format!("MySQL pool acquire failed: {e}")),
                    },
                    Err(e) => Err(format!("MySQL: {e}")),
                }
            },
            EngineType::Sqlite => {
                match SqlitePoolOptions::new().max_connections(5).connect(dsn_str).await {
                    Ok(pool) => match pool.acquire().await {
                        Ok(_conn) => {
                            info!("Connected to SQLite");
                            Ok(Self::Sqlite(pool))
                        },
                        Err(e) => Err(format!("SQLite pool acquire failed: {e}")),
                    },
                    Err(e) => Err(format!("SQLite: {e}")),
                }
            },
        }
    }

    pub async fn close(self) {
        match self {
            Self::Pg(pool) => pool.close().await,
            Self::Mysql(pool) => pool.close().await,
            Self::Sqlite(pool) => pool.close().await,
            Self::Disconnected => {},
        }
    }

    #[allow(dead_code)]
    pub const fn is_connected(&self) -> bool {
        !matches!(self, Self::Disconnected)
    }
}
