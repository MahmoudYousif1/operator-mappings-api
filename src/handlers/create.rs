use crate::app_state::AppState;
use crate::app_state::persistence::save_to_disk;
use crate::operators::crud_operations::create::create_operator;
use crate::utils::error_responses::ErrorResponse;
use crate::utils::models::CreateOperator;
use actix_web::{HttpResponse, web};
use serde_json::json;

pub async fn handle_create_operator(
    app_data: web::Data<AppState>,
    new_op: web::Json<CreateOperator>,
) -> Result<HttpResponse, ErrorResponse> {
    let operator = create_operator(&app_data, new_op.into_inner())?;
    save_to_disk(&app_data, &app_data.mappings_file_path)
        .await
        .map_err(|_| ErrorResponse::InternalError)?;

    Ok(HttpResponse::Created().json(json!({
        "status":   "successfully created",
        "operator": *operator
    })))
}
