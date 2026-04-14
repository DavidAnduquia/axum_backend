use std::sync::Arc;

use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use sqlx::PgPool;

/// Wrapper ligero para compartir el `PgPool` y entregar conexiones de SeaORM
/// construidas bajo demanda (lazy) para reducir overhead inicial.
#[derive(Clone, Debug)]
pub struct DbExecutor {
    pool: Arc<PgPool>,
}

impl DbExecutor {
    /// Crea un `DbExecutor` a partir de un `Arc<PgPool>` existente.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Consume el `PgPool` y genera internamente el `Arc`.
    pub fn from_pool(pool: PgPool) -> Self {
        Self::new(Arc::new(pool))
    }

    /// Devuelve una referencia al pool de SQLx.
    pub fn pool(&self) -> &PgPool {
        self.pool.as_ref()
    }

    /// Devuelve un `Arc` clonado del pool para quienes necesiten ownership.

    /// Crea una conexión de SeaORM bajo demanda. Cada llamada clona el pool
    /// internamente, pero evita mantener la conexión pre-construida en memoria.
    pub fn connection(&self) -> DatabaseConnection {
        SqlxPostgresConnector::from_sqlx_postgres_pool((*self.pool).clone())
    }
}
