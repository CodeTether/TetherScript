//! Actix request snapshots and tetherscript request maps.

use std::collections::HashMap;

use ::actix_web::{web, HttpRequest};

use crate::value::Value;

pub(crate) struct RequestData {
    method: String,
    path: String,
    query: String,
    headers: HashMap<String, String>,
    params: HashMap<String, String>,
    body: Vec<u8>,
}

impl RequestData {
    pub fn capture(request: &HttpRequest, body: &web::Bytes) -> Self {
        let headers = request.headers().iter().map(|(name, value)| {
            (
                name.to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        });
        let params = request
            .match_info()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()));
        Self {
            method: request.method().to_string(),
            path: request.path().to_string(),
            query: request.query_string().to_string(),
            headers: headers.collect(),
            params: params.collect(),
            body: body.to_vec(),
        }
    }

    pub fn into_value(self) -> Value {
        super::request_value::from_parts(
            self.method,
            self.path,
            self.query,
            self.headers,
            self.params,
            self.body,
        )
    }
}
