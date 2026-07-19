//! Blocking-pool execution with one cached runtime per pool thread.

use ::actix_web::{web, HttpRequest, HttpResponse};

use super::error::ActixPluginError;
use super::request::RequestData;
use super::response::ResponseData;
use super::state::RouteState;

pub(crate) async fn handle(
    request: HttpRequest,
    body: web::Bytes,
    state: web::Data<RouteState>,
) -> Result<HttpResponse, ActixPluginError> {
    let request = RequestData::capture(&request, &body);
    let state = state.get_ref().clone();
    let response = web::block(move || execute(&state, request))
        .await
        .map_err(|error| ActixPluginError::Blocking(error.to_string()))??;
    Ok(response.into_http())
}

fn execute(state: &RouteState, request: RequestData) -> Result<ResponseData, ActixPluginError> {
    super::reload::refresh(state);
    let call = super::cache::call(state, &[request.into_value()])?;
    ResponseData::from_value(call.value)
}
