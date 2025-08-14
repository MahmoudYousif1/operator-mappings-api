use actix_web::{HttpResponse, Result, web};

use crate::{
    app_state::model::AppState,
    operators::group_operators_by_iso3::get_country_operator_mapping,
    utils::{
        error_responses::ErrorResponse,
        models::{OperatorValidationError, ReadQuery},
    },
};

pub async fn handle_get_country_mapping(
    app_state: web::Data<AppState>,
    query: web::Query<ReadQuery>,
) -> Result<HttpResponse, ErrorResponse> {
    let iso3 = query.iso3.as_ref().ok_or_else(|| {
        ErrorResponse::Validation(OperatorValidationError::FieldValidationError {
            field: "iso3".into(),
            message: "iso3 query parameter is required".to_string(),
            received: None,
        })
    })?;
    let grouped_operators_by_iso3 = get_country_operator_mapping(&app_state, iso3)?;
    Ok(HttpResponse::Ok().json(grouped_operators_by_iso3))
}
