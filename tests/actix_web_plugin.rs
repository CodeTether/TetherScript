#![cfg(feature = "actix-web")]

#[path = "actix_web_plugin/authority.rs"]
mod authority;
#[path = "actix_web_plugin/db_route.rs"]
mod db_route;
#[path = "actix_web_plugin/db_value.rs"]
mod db_value;
#[path = "actix_web_plugin/hot_reload.rs"]
mod hot_reload;
#[path = "actix_web_plugin/route.rs"]
mod route;
