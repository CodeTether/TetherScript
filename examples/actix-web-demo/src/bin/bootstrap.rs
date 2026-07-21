//! Create the isolated demo database and import the remote dataset.

#[path = "../bootstrap_database.rs"]
mod bootstrap_database;
#[path = "../dataset.rs"]
mod dataset;
#[path = "../db_config.rs"]
mod db_config;
#[path = "../db_pool.rs"]
mod db_pool;
#[path = "../import.rs"]
mod import;
#[path = "../import_sql.rs"]
mod import_sql;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    bootstrap_database::ensure().await?;
    let pool = db_pool::create().await?;
    let current: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    if current != db_config::DATABASE_NAME {
        return Err(format!("refusing schema changes in database `{current}`").into());
    }
    import::schema(&pool).await?;
    let points = dataset::fetch()?;
    let imported = import::replace(&pool, &points).await?;
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM country_gdp")
        .fetch_one(&pool)
        .await?;
    println!("imported {imported} World Bank records; database={current}; rows={rows}");
    Ok(())
}
