//! Conversion of script response maps into Actix responses.

use ::actix_web::{
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    HttpResponse,
};

use crate::value::Value;

use super::error::ActixPluginError;

pub(crate) struct ResponseData {
    pub(super) status: u16,
    pub(super) headers: Vec<(HeaderName, HeaderValue)>,
    pub(super) body: Vec<u8>,
}

impl ResponseData {
    pub fn from_value(value: Value) -> Result<Self, ActixPluginError> {
        super::response_parse::parse(value)
    }

    pub fn into_http(self) -> HttpResponse {
        let mut response = HttpResponse::build(StatusCode::from_u16(self.status).unwrap());
        for (name, value) in self.headers {
            response.append_header((name, value));
        }
        response.body(self.body)
    }
}
