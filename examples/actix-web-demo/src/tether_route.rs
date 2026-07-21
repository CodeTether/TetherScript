//! File-backed tetherscript database controller registration.

use std::rc::Rc;

use actix_web::http::Method;
use tetherscript::actix_web::ActixPlugin;
use tetherscript::database::DatabaseAuthority;
use tetherscript::plugin::PluginHost;
use tokio::runtime::Handle;

use crate::db_authority::SqlxQueryHandler;
use crate::db_pool::DbPool;

const CONTROLLER: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/controllers/tether_route.tether"
);

pub fn build(pool: DbPool, runtime: Handle) -> ActixPlugin {
    ActixPlugin::from_file("/tether/country/{code}", Method::GET, CONTROLLER)
        .plugin_name("country-database-controller")
        .host_factory(move || {
            let mut host = PluginHost::new();
            let handler = SqlxQueryHandler::new(pool.clone(), runtime.clone());
            let authority = DatabaseAuthority::new(handler);
            host.grant("db", Rc::new(authority));
            host
        })
        .build()
        .expect("tetherscript database controller should load")
}
