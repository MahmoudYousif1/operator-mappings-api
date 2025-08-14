use std::sync::Arc;

use crate::{
    app_state::model::AppState,
    utils::{
        error_responses::ErrorResponse,
        models::{Operator, QueryType, SubscriberIdKind},
        validations::validate_digits,
    },
};

fn common_prefix_length(str1: &str, str2: &str) -> usize {
    str1.chars()
        .zip(str2.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .count()
}

fn find_operator_with_longest_prefix_match<'a>(
    operator_list: &'a [Operator],
    extract_field: impl Fn(&Operator) -> Option<&Vec<String>>,
    lookup_value: &str,
) -> Option<&'a Operator> {
    operator_list
        .iter()
        .filter_map(|op| {
            extract_field(op).and_then(|prefixes| {
                let best = prefixes
                    .iter()
                    .map(|p| common_prefix_length(p, lookup_value))
                    .max()?;
                if best > 0 { Some((best, op)) } else { None }
            })
        })
        .max_by_key(|(len, _)| *len)
        .map(|(_, op)| op)
}

fn find_operator_by_tadig_exact_match<'a>(
    operator_list: &'a [Operator],
    tadig_code: &str,
) -> Option<&'a Operator> {
    operator_list.iter().find(|op| {
        op.tadig
            .as_ref()
            .is_some_and(|codes| codes.iter().any(|c| c == tadig_code))
    })
}

pub fn lookup_operator_by_query(
    state: &AppState,
    query_text: &str,
    query_type: QueryType,
) -> Result<Option<Arc<Operator>>, ErrorResponse> {
    let guard = state
        .operators
        .read()
        .map_err(|_| ErrorResponse::InternalError)?;
    let flat: Vec<Operator> = guard.iter().map(|arc| (**arc).clone()).collect();

    let found = match query_type {
        QueryType::Imsi => {
            let imsi = query_text.trim();
            validate_digits(SubscriberIdKind::Imsi, "imsi", imsi)
                .map_err(ErrorResponse::Validation)?;
            find_operator_with_longest_prefix_match(&flat, |op| op.e212.as_ref(), imsi)
        }
        QueryType::Msisdn => {
            let msisdn = query_text.trim();
            validate_digits(SubscriberIdKind::Msisdn, "msisdn", msisdn)
                .map_err(ErrorResponse::Validation)?;
            find_operator_with_longest_prefix_match(&flat, |op| op.e164.as_ref(), msisdn)
        }
        QueryType::Tadig => {
            let code = query_text.trim().to_uppercase();
            find_operator_by_tadig_exact_match(&flat, &code)
        }
        QueryType::Iso3 => None,
    };

    Ok(found.map(|op| Arc::new(op.clone())))
}
