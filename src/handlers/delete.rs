use crate::{
    app_state::{model::AppState, persistence::save_to_disk},
    operators::crud_operations::delete::delete_operator,
    utils::error_responses::ErrorResponse,
};
use actix_web::{HttpResponse, web};

pub async fn handle_delete_operator(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ErrorResponse> {
    let tadig = path.into_inner();

    delete_operator(&app_state, &tadig)?;

    save_to_disk(&app_state, &app_state.mappings_file_path)
        .await
        .map_err(|_| ErrorResponse::InternalError)?;

    Ok(HttpResponse::NoContent().finish())
}
