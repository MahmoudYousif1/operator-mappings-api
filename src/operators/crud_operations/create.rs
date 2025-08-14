use crate::app_state::model::AppState;
use crate::utils::{
    error_responses::ErrorResponse,
    models::{CreateOperator, Operator, OperatorDuplicateChecker},
    validations,
};
use std::sync::Arc;

fn validate_create_input(
    operators: &[Operator],
    input: &CreateOperator,
    country: &str,
) -> Result<(), ErrorResponse> {
    let _ = country;
    let collected_existing_operator_data = OperatorDuplicateChecker::from_operators(operators);

    validations::validate_unique_imsi_codes(
        input.e212.as_deref(),
        &collected_existing_operator_data.imsi_prefixes,
    )?;
    validations::validate_unique_msisdn_codes(
        input.e164.as_deref(),
        &collected_existing_operator_data.msisdn_prefixes,
    )?;
    validations::validate_unique_operator_name(
        &input.name,
        &collected_existing_operator_data.operator_names,
    )?;
    let tadig_list = input.tadig.clone().unwrap_or_default();
    validations::validate_unique_tadig_codes(
        &tadig_list,
        &collected_existing_operator_data
            .tadig_codes
            .iter()
            .map(|&s| s.to_string())
            .collect(),
    )
}

pub fn create_operator(
    app_data: &AppState,
    input: CreateOperator,
) -> Result<Arc<Operator>, ErrorResponse> {
    let country = validations::format_country_name(&input.country)?;

    let guard_read = app_data
        .operators
        .read()
        .map_err(|_| ErrorResponse::InternalError)?;
    let cloned_operator_list: Vec<Operator> =
        guard_read.iter().map(|arc| (**arc).clone()).collect();
    let country_operator = validations::find_operator_by_country(&cloned_operator_list, &country)?;
    validate_create_input(&cloned_operator_list, &input, &country)?;
    drop(guard_read);

    let tadig_list = input.tadig.clone().unwrap_or_default();
    let iso2 = country_operator.iso2.clone();
    let iso3 = country_operator.iso3.clone();
    let new_op = Operator {
        country,
        e164: input.e164.clone(),
        e212: input.e212.clone(),
        iso2,
        iso3,
        name: input.name.clone(),
        realm: input.realm.clone(),
        tadig: Some(tadig_list),
    };

    let new_arc = Arc::new(new_op.clone());

    {
        let mut guard_write = app_data
            .operators
            .write()
            .map_err(|_| ErrorResponse::InternalError)?;
        guard_write.push(Arc::new(new_op));
    }

    Ok(new_arc)
}
