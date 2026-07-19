//! Actix Web routes backed by sandboxed tetherscript hooks.
//!
//! [`ActixPlugin`] turns a script function into an Actix route. Each blocking
//! worker caches its own loaded plugin, while a host factory can grant Rust
//! capabilities backed by database pools or application services.
//!
//! # Example
//!
//! ```rust,no_run
//! use actix_web::{http::Method, App};
//! use tetherscript::actix_web::ActixPlugin;
//!
//! let source = r#"fn handle(request) {
//!     let response = map()
//!     response.status = 200
//!     response.body = "hello from tetherscript"
//!     return response
//! }"#;
//! let plugin = ActixPlugin::builder("/hello", Method::GET, source)
//!     .build()
//!     .unwrap();
//! let _app = App::new().configure(|config| plugin.configure(config));
//! ```

mod builder;
mod builder_build;
mod builder_new;
mod cache;
mod error;
mod execute;
mod reload;
mod request;
mod request_value;
mod response;
mod response_headers;
mod response_parse;
mod route;
mod script;
mod source;
mod state;
mod validate;

pub use builder::ActixPluginBuilder;
pub use error::ActixPluginError;
pub use route::ActixPlugin;
