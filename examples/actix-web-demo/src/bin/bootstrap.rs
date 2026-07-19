//! Create the isolated demo database and import the remote dataset.

#[path = "../dataset.rs"]
mod dataset;
#[path = "../db_config.rs"]
mod db_config;
#[path = "../import.rs"]
mod import;

use postgres::NoTls;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut admin = db_config::config("postgres")?.connect(NoTls)?;
    let exists: bool = admin
        .query_one(
            "SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)",
            &[&db_config::DATABASE_NAME],
        )?
        .get(0);
    if !exists {
        admin.batch_execute("CREATE DATABASE tetherscript_actix_demo")?;
        println!("created database {}", db_config::DATABASE_NAME);
    } else {
        println!("database {} already exists", db_config::DATABASE_NAME);
    }
    drop(admin);

    let mut client = db_config::config(db_config::DATABASE_NAME)?.connect(NoTls)?;
    let current: String = client.query_one("SELECT current_database()", &[])?.get(0);
    if current != db_config::DATABASE_NAME {
        return Err(format!("refusing schema changes in database `{current}`").into());
    }
    import::schema(&mut client)?;
    let points = dataset::fetch()?;
    let imported = import::replace(&mut client, &points)?;
    let rows: i64 = client
        .query_one("SELECT COUNT(*) FROM country_gdp", &[])?
        .get(0);
    println!("imported {imported} World Bank records; database={current}; rows={rows}");
    Ok(())
}
