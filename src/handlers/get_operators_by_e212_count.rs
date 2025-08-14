use crate::{
    app_state::model::AppState,
    operators::get_operators_by_e212_count::get_operators_grouped_by_e212_count,
    utils::error_responses::ErrorResponse,
};
use actix_web::{HttpResponse, Result, web};

pub async fn handle_get_operators_by_e212_count(
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, ErrorResponse> {
    Ok(HttpResponse::Ok().json(get_operators_grouped_by_e212_count(&app_state)?))
}
