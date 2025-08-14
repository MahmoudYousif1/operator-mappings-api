use crate::{
    app_state::model::AppState, operators::find_roaming_partners::find_roaming_partners,
    utils::error_responses::ErrorResponse, utils::models::ReadQuery,
};
use actix_web::{HttpResponse, Result, web};

pub async fn handle_find_roaming_partners(
    app_state: web::Data<AppState>,
    query: web::Query<ReadQuery>,
) -> Result<HttpResponse, ErrorResponse> {
    let this_tadig: String = query.tadig.clone().unwrap_or_default();
    let tadig: &str = this_tadig.trim();

    if tadig.is_empty() {
        return Err(ErrorResponse::NotFound {
            field: "tadig".to_string(),
            received: this_tadig,
            expected: "a non-empty TADIG".to_string(),
        });
    }
    Ok(HttpResponse::Ok().json(find_roaming_partners(&app_state, tadig)?))
}
