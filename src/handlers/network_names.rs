use actix_web::{HttpResponse, Result, web};

use crate::{
    app_state::model::AppState, operators::network_names::retrieve_network_names,
    utils::error_responses::ErrorResponse,
};

pub async fn handle_network_names(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ErrorResponse> {
    let network_names = retrieve_network_names(&app_state)?;
    Ok(HttpResponse::Ok().json(network_names))
}
