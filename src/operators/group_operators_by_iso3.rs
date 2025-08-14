use crate::{
    app_state::model::AppState,
    utils::{
        error_responses::ErrorResponse,
        models::{GroupOperatorsSummary, Operator},
        validations::validate_iso3_code,
    },
};

pub fn get_country_operator_mapping(
    state: &AppState,
    iso3_param: &str,
) -> Result<GroupOperatorsSummary, ErrorResponse> {
    validate_iso3_code("iso3", iso3_param).map_err(ErrorResponse::Validation)?;

    let guard = state
        .operators
        .read()
        .map_err(|_| ErrorResponse::InternalError)?;

    let matches: Vec<Operator> = guard
        .iter()
        .filter(|arc_op| arc_op.iso3.eq_ignore_ascii_case(iso3_param))
        .map(|arc_op| (**arc_op).clone())
        .collect();

    if matches.is_empty() {
        return Err(ErrorResponse::NotFound {
            field: "iso3".to_string(),
            received: iso3_param.to_string(),
            expected: "an existing ISO3 code".to_string(),
        });
    }

    Ok(GroupOperatorsSummary {
        iso3: iso3_param.to_uppercase(),
        total: matches.len(),
        operators: matches,
    })
}
