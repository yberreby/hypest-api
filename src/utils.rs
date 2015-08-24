use prelude::*;

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

pub struct AppDb;
impl Key for AppDb { type Value = PostgresPool; }

// Helper methods
pub fn setup_connection_pool(connection_str: &str, pool_size: u32) -> PostgresPool {
    let manager = PostgresConnectionManager::new(connection_str, postgres::SslMode::None).unwrap();
    let config = r2d2::Config::builder().pool_size(pool_size).build();
    r2d2::Pool::new(config, manager).unwrap()
}
