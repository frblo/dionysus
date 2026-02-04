use sqlx::PgPool;

/// Database connection
///
/// Cheap-to-clone handle to database
#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Create a new [`Db`] from an already-configured pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
