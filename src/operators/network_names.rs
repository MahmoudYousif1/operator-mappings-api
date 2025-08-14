use crate::{
    app_state::AppState,
    utils::{error_responses::ErrorResponse, validations::format_network_name},
};

pub fn retrieve_network_names(app_state: &AppState) -> Result<Vec<String>, ErrorResponse> {
    let guard = app_state
        .operators
        .read()
        .map_err(|_| ErrorResponse::InternalError)?;

    let mut names: Vec<String> = guard
        .iter()
        .filter_map(|op| op.name.as_ref())
        .map(|n| format_network_name(n))
        .collect();

    names.sort_unstable();
    names.dedup();

    if names.is_empty() {
        return Err(ErrorResponse::NotFound {
            field: "Operator names".to_string(),
            received: "".to_string(),
            expected: "at least one operator name".to_string(),
        });
    }

    Ok(names)
}
