//! World Bank GDP dataset download and typed extraction.

pub const SOURCE_URL: &str = "https://api.worldbank.org/v2/country/all/indicator/NY.GDP.MKTP.CD?format=json&per_page=400&date=2022";

pub struct DataPoint {
    pub code: String,
    pub name: String,
    pub year: i32,
    pub gdp_usd: f64,
}

pub fn fetch() -> Result<Vec<DataPoint>, Box<dyn std::error::Error>> {
    let payload: serde_json::Value = ureq::get(SOURCE_URL).call()?.into_json()?;
    let rows = payload
        .get(1)
        .and_then(serde_json::Value::as_array)
        .ok_or("World Bank response did not contain a data array")?;
    let mut points = Vec::new();
    for row in rows {
        let Some(code) = row.get("countryiso3code").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(name) = row
            .pointer("/country/value")
            .and_then(|value| value.as_str())
        else {
            continue;
        };
        let Some(year) = row.get("date").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(gdp_usd) = row.get("value").and_then(|value| value.as_f64()) else {
            continue;
        };
        if code.len() == 3 {
            points.push(DataPoint {
                code: code.into(),
                name: name.into(),
                year: year.parse()?,
                gdp_usd,
            });
        }
    }
    Ok(points)
}
