//! Isolated demo schema and idempotent dataset import.

use postgres::Client;

use crate::dataset::{DataPoint, SOURCE_URL};

pub fn schema(client: &mut Client) -> Result<(), postgres::Error> {
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS country_gdp (
            country_code TEXT PRIMARY KEY CHECK (char_length(country_code) = 3),
            country_name TEXT NOT NULL,
            year INTEGER NOT NULL,
            gdp_usd DOUBLE PRECISION NOT NULL CHECK (gdp_usd >= 0),
            source_url TEXT NOT NULL,
            imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS country_gdp_rank_idx ON country_gdp (gdp_usd DESC);",
    )
}

pub fn replace(client: &mut Client, points: &[DataPoint]) -> Result<u64, postgres::Error> {
    let mut transaction = client.transaction()?;
    transaction.execute("DELETE FROM country_gdp WHERE year = $1", &[&2022_i32])?;
    let statement = transaction.prepare(
        "INSERT INTO country_gdp
         (country_code, country_name, year, gdp_usd, source_url)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (country_code) DO UPDATE SET
           country_name = EXCLUDED.country_name,
           year = EXCLUDED.year,
           gdp_usd = EXCLUDED.gdp_usd,
           source_url = EXCLUDED.source_url,
           imported_at = NOW()",
    )?;
    let mut imported = 0;
    for point in points {
        imported += transaction.execute(
            &statement,
            &[
                &point.code,
                &point.name,
                &point.year,
                &point.gdp_usd,
                &SOURCE_URL,
            ],
        )?;
    }
    transaction.commit()?;
    Ok(imported)
}
