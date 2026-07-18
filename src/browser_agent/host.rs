//! Native HTTP action host for stateful agent browser capabilities.

mod dispatch;
mod fetch;
mod fetch_response;
mod focus;
mod health;
#[cfg(test)]
mod input_tests;
mod interact;
mod interact_focus;
mod interact_value;
mod keyboard;
mod nav;
mod nav_load;
mod png;
mod png_chunk;
mod png_zlib;
mod query;
mod request;
mod request_headers;
mod response;
mod screenshot;
mod scroll;
mod scroll_target;
#[cfg(test)]
mod scroll_tests;
mod server;
mod snapshot;
mod state;
mod url;
mod value;
mod value_optional;
mod viewport;
mod visible_text;
#[cfg(test)]
mod visible_text_tests;
mod wait;
mod wait_poll;
mod wait_selector;
#[cfg(test)]
mod wait_tests;

/// Serve native browser action envelopes over blocking HTTP.
///
/// # Arguments
///
/// * `address` - Socket address such as `127.0.0.1:41707`.
///
/// # Returns
///
/// Returns after a `stop` action, or an error when binding or serving fails.
///
/// # Errors
///
/// Returns an error when the listener cannot bind or a connection fails.
pub fn serve(address: &str) -> Result<(), String> {
    server::serve(address)
}
