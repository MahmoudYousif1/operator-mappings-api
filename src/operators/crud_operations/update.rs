use crate::{
    app_state::model::AppState,
    utils::{
        error_responses::ErrorResponse,
        models::{CreateOperator, Operator, OperatorDuplicateChecker, PatchOperator},
        validations::{self, determine_updated_codes, validate_iso_fields, validate_patch_fields},
    },
};
use std::{collections::HashSet, sync::Arc};

fn update_validations(
    all_operators: &[Operator],
    target_index: usize,
    new_operator: &CreateOperator,
    formatted_country: &str,
) -> Result<(), ErrorResponse> {
    validations::find_operator_by_country(all_operators, formatted_country)?;

    let mut duplicate_field_checker = OperatorDuplicateChecker::from_operators(all_operators);
    duplicate_field_checker.exclude(&all_operators[target_index]);

    validations::validate_unique_imsi_codes(
        new_operator.e212.as_deref(),
        &duplicate_field_checker.imsi_prefixes,
    )?;
    validations::validate_unique_msisdn_codes(
        new_operator.e164.as_deref(),
        &duplicate_field_checker.msisdn_prefixes,
    )?;
    validations::validate_unique_operator_name(
        &new_operator.name,
        &duplicate_field_checker.operator_names,
    )?;

    let updated_tadig_codes = new_operator.tadig.as_deref().unwrap_or_default();
    let existing_tadig_set: HashSet<String> = duplicate_field_checker
        .tadig_codes
        .iter()
        .map(ToString::to_string)
        .collect();
    validations::validate_unique_tadig_codes(updated_tadig_codes, &existing_tadig_set)?;
    Ok(())
}

pub fn update_operator_by_put(
    state: &AppState,
    target_tadig: &str,
    updated_fields: CreateOperator,
) -> Result<Arc<Operator>, ErrorResponse> {
    let validated_country = validations::format_country_name(&updated_fields.country)?;
    let operator_list: Vec<Arc<Operator>> = {
        let read_guard = state
            .operators
            .read()
            .map_err(|_| ErrorResponse::InternalError)?;
        read_guard.iter().cloned().collect()
    };

    let operator_index = operator_list
        .iter()
        .position(|op| op.has_tadig(target_tadig))
        .ok_or_else(|| ErrorResponse::NotFound {
            field: "tadig".to_string(),
            received: target_tadig.to_string(),
            expected: "an existing TADIG code".to_string(),
        })?;

    let existing_operators: Vec<Operator> =
        operator_list.iter().map(|arc| (**arc).clone()).collect();
    update_validations(
        &existing_operators,
        operator_index,
        &updated_fields,
        &validated_country,
    )?;

    let (matched_country_iso2, matched_country_iso3) = {
        let country_operator =
            validations::find_operator_by_country(&existing_operators, &validated_country)?;
        (country_operator.iso2.clone(), country_operator.iso3.clone())
    };

    let fully_updated_operator = Operator {
        country: validated_country,
        e164: updated_fields.e164,
        e212: updated_fields.e212,
        iso2: matched_country_iso2,
        iso3: matched_country_iso3,
        name: updated_fields.name,
        realm: updated_fields.realm,
        tadig: Some(updated_fields.tadig.unwrap_or_default()),
    };

    {
        let mut write_guard = state
            .operators
            .write()
            .map_err(|_| ErrorResponse::InternalError)?;
        write_guard[operator_index] = Arc::new(fully_updated_operator.clone());
    }

    Ok(Arc::new(fully_updated_operator))
}

pub fn update_operator_by_patch(
    state: &AppState,
    target_tadig: &str,
    patch_data: PatchOperator,
) -> Result<Arc<Operator>, ErrorResponse> {
    validate_iso_fields(&patch_data)?;

    let mut operator_store = state
        .operators
        .write()
        .map_err(|_| ErrorResponse::InternalError)?;
    let operator_index = operator_store
        .iter()
        .position(|op| op.has_tadig(target_tadig))
        .ok_or_else(|| ErrorResponse::NotFound {
            field: "tadig".to_string(),
            received: target_tadig.to_string(),
            expected: "an existing TADIG code".to_string(),
        })?;

    let operator_store_clone: Vec<Arc<Operator>> = operator_store.clone();
    let mut cloned_plain_operators: Vec<Operator> = operator_store_clone
        .iter()
        .map(|arc| (**arc).clone())
        .collect();
    cloned_plain_operators.remove(operator_index);
    let duplicate_field_checker = OperatorDuplicateChecker::from_operators(&cloned_plain_operators);

    let operator_entry = &mut operator_store[operator_index];
    let mutable_operator = Arc::make_mut(operator_entry);

    let (resolved_country, matched_country_iso2, matched_country_iso3) =
        determine_updated_codes(&operator_store_clone, &patch_data, mutable_operator)?;
    validations::format_country_name(&resolved_country)?;
    validate_patch_fields(&duplicate_field_checker, &patch_data)?;

    mutable_operator.country = resolved_country;
    mutable_operator.iso2 = matched_country_iso2;
    mutable_operator.iso3 = matched_country_iso3;
    if let Some(e212_list) = patch_data.e212 {
        mutable_operator.e212 = Some(e212_list);
    }
    if let Some(e164_list) = patch_data.e164 {
        mutable_operator.e164 = Some(e164_list);
    }
    if let Some(name_opt) = patch_data.name {
        mutable_operator.name = name_opt;
    }
    if let Some(realm) = patch_data.realm {
        mutable_operator.realm = Some(realm);
    }
    if let Some(tadig) = patch_data.tadig {
        mutable_operator.tadig = Some(tadig);
    }

    Ok(operator_store[operator_index].clone())
}
