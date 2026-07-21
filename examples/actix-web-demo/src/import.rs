//! Isolated demo schema and idempotent dataset import.

use crate::dataset::{DataPoint, SOURCE_URL};
use crate::db_pool::DbPool;
use crate::import_sql::{INDEX, SCHEMA, UPSERT};

pub async fn schema(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(SCHEMA).execute(pool).await?;
    sqlx::query(INDEX).execute(pool).await?;
    Ok(())
}

pub async fn replace(pool: &DbPool, points: &[DataPoint]) -> Result<u64, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    sqlx::query("DELETE FROM country_gdp WHERE year = $1")
        .bind(2022_i32)
        .execute(&mut *transaction)
        .await?;
    let mut imported = 0;
    for point in points {
        imported += sqlx::query(UPSERT)
            .bind(&point.code)
            .bind(&point.name)
            .bind(point.year)
            .bind(point.gdp_usd)
            .bind(SOURCE_URL)
            .execute(&mut *transaction)
            .await?
            .rows_affected();
    }
    transaction.commit().await?;
    Ok(imported)
}
