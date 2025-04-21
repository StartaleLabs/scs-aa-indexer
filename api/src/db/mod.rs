use sqlx::postgres::PgPoolOptions;
use std::env;

pub type Db = sqlx::Pool<sqlx::Postgres>;

pub async fn connect() -> Result<Db, sqlx::Error> {
    let db_url = env::var("TIMESCALE_DB_URL")
        .expect("TIMESCALE_DB_URL must be set in .env");
    
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
}
