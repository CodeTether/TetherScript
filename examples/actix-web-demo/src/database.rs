//! Shared ranked-country PostgreSQL query.

use serde::Serialize;
use sqlx::postgres::PgRow;
use sqlx::Row;

use crate::db_pool::DbPool;

#[derive(Serialize)]
pub struct CountryRecord {
    pub code: String,
    pub name: String,
    pub year: i32,
    pub gdp_usd: f64,
    pub rank: i64,
    pub dataset_rows: i64,
}

const SQL: &str = "WITH ranked AS (SELECT country_code, country_name, year, gdp_usd, \
    RANK() OVER (ORDER BY gdp_usd DESC) AS rank, COUNT(*) OVER () AS dataset_rows \
    FROM country_gdp) SELECT country_code, country_name, year, gdp_usd, rank, \
    dataset_rows FROM ranked WHERE country_code = UPPER($1)";

pub async fn country(pool: &DbPool, code: &str) -> Result<Option<CountryRecord>, String> {
    sqlx::query(SQL)
        .bind(code)
        .fetch_optional(pool)
        .await
        .map_err(|error| error.to_string())?
        .map(decode)
        .transpose()
        .map_err(|error| error.to_string())
}

fn decode(row: PgRow) -> Result<CountryRecord, sqlx::Error> {
    Ok(CountryRecord {
        code: row.try_get("country_code")?,
        name: row.try_get("country_name")?,
        year: row.try_get("year")?,
        gdp_usd: row.try_get("gdp_usd")?,
        rank: row.try_get("rank")?,
        dataset_rows: row.try_get("dataset_rows")?,
    })
}
