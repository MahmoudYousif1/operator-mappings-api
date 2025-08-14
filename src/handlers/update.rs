use actix_web::{HttpResponse, Result, web};
use serde_json::json;

use crate::{
    app_state::{model::AppState, persistence::save_to_disk},
    operators::crud_operations::update::{update_operator_by_patch, update_operator_by_put},
    utils::{
        error_responses::ErrorResponse,
        models::{CreateOperator, PatchOperator},
    },
};

pub async fn handle_update_by_put(
    app_state: web::Data<AppState>,
    tadig_path: web::Path<String>,
    operator_payload: web::Json<CreateOperator>,
) -> Result<HttpResponse, ErrorResponse> {
    let tadig_key = tadig_path.into_inner();
    let updated_operator =
        update_operator_by_put(&app_state, &tadig_key, operator_payload.into_inner())?;

    save_to_disk(&app_state, &app_state.mappings_file_path)
        .await
        .map_err(|_| ErrorResponse::InternalError)?;

    Ok(HttpResponse::Ok().json(json!({
        "status":   "successfully_updated",
        "operator": *updated_operator
    })))
}

pub async fn handle_update_by_patch(
    app_state: web::Data<AppState>,
    tadig_path: web::Path<String>,
    patch_payload: web::Json<PatchOperator>,
) -> Result<HttpResponse, ErrorResponse> {
    let tadig_key = tadig_path.into_inner();
    let updated_operator =
        update_operator_by_patch(&app_state, &tadig_key, patch_payload.into_inner())?;

    save_to_disk(&app_state, &app_state.mappings_file_path)
        .await
        .map_err(|_| ErrorResponse::InternalError)?;

    Ok(HttpResponse::Ok().json(json!({
        "status":   "successfully_updated",
        "operator": *updated_operator
    })))
}
