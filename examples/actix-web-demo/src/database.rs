//! Shared ranked-country PostgreSQL query.

use serde::Serialize;

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

pub fn country(pool: &DbPool, code: &str) -> Result<Option<CountryRecord>, String> {
    let mut connection = pool.get().map_err(|error| error.to_string())?;
    let row = connection
        .query_opt(
            "WITH ranked AS (SELECT country_code, country_name, year, gdp_usd, \
             RANK() OVER (ORDER BY gdp_usd DESC) AS rank, COUNT(*) OVER () AS total \
             FROM country_gdp) SELECT country_code, country_name, year, gdp_usd, rank, total \
             FROM ranked WHERE country_code = UPPER($1)",
            &[&code],
        )
        .map_err(|error| error.to_string())?;
    Ok(row.map(|row| CountryRecord {
        code: row.get(0),
        name: row.get(1),
        year: row.get(2),
        gdp_usd: row.get(3),
        rank: row.get(4),
        dataset_rows: row.get(5),
    }))
}
