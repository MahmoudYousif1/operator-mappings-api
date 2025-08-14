use crate::{
    app_state::AppState,
    utils::{error_responses::ErrorResponse, models::Operator},
};
use std::sync::Arc;

pub fn delete_operator(state: &AppState, tadig_code: &str) -> Result<Arc<Operator>, ErrorResponse> {
    let mut operators_collection = state
        .operators
        .write()
        .map_err(|_| ErrorResponse::InternalError)?;

    if let Some((operator_index, _)) =
        operators_collection
            .iter()
            .enumerate()
            .find(|(_, operator)| {
                operator.tadig.as_ref().is_some_and(|tadig_codes| {
                    tadig_codes
                        .iter()
                        .any(|cur_tadig_code| cur_tadig_code == tadig_code)
                })
            })
    {
        let removed_operator = operators_collection.swap_remove(operator_index);
        Ok(removed_operator)
    } else {
        Err(ErrorResponse::NotFound {
            field: "tadig".to_string(),
            received: tadig_code.to_string(),
            expected: "an existing TADIG code".to_string(),
        })
    }
}
