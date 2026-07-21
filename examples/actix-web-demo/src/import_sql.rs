//! SQL statements used only by the isolated dataset bootstrap.

pub(crate) const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS country_gdp (
    country_code TEXT PRIMARY KEY CHECK (char_length(country_code) = 3),
    country_name TEXT NOT NULL,
    year INTEGER NOT NULL,
    gdp_usd DOUBLE PRECISION NOT NULL CHECK (gdp_usd >= 0),
    source_url TEXT NOT NULL,
    imported_at TIMESTAMPTZ NOT NULL DEFAULT NOW())";

pub(crate) const INDEX: &str =
    "CREATE INDEX IF NOT EXISTS country_gdp_rank_idx ON country_gdp (gdp_usd DESC)";

pub(crate) const UPSERT: &str = "INSERT INTO country_gdp
    (country_code, country_name, year, gdp_usd, source_url)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (country_code) DO UPDATE SET
    country_name = EXCLUDED.country_name, year = EXCLUDED.year,
    gdp_usd = EXCLUDED.gdp_usd, source_url = EXCLUDED.source_url,
    imported_at = NOW()";
