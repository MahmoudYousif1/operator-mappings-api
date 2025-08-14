use crate::utils::{
    error_responses::ErrorResponse,
    models::{
        Operator, OperatorDuplicateChecker, OperatorValidationError, PatchOperator,
        SubscriberIdKind,
    },
};
use once_cell::sync::Lazy as SyncLazy;
use regex::Regex;
use std::{borrow::Borrow, collections::HashSet, fmt::Display, hash::Hash, sync::Arc};

pub static TADIG_RE: SyncLazy<Regex> =
    SyncLazy::new(|| Regex::new(r"^[A-Za-z0-9_]{5,9}$").unwrap());

pub fn validate_non_empty<'a>(
    field_name: &'a str,
    field_value: &'a str,
) -> Result<&'a str, OperatorValidationError> {
    let trimmed = field_value.trim();
    if trimmed.is_empty() {
        Err(OperatorValidationError::FieldValidationError {
            field: field_name.to_string(),
            message: format!("{} cannot be empty", field_name),
            received: Some(trimmed.to_string()),
        })
    } else {
        Ok(trimmed)
    }
}

pub fn validate_digits(
    kind: SubscriberIdKind,
    field_name: &str,
    val: &str,
) -> Result<(), OperatorValidationError> {
    if val.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(kind.error_kind(field_name.to_string(), val.to_string()))
    }
}

pub fn validate_characters_and_spaces(
    field_name: &str,
    field_value: &str,
) -> Result<(), OperatorValidationError> {
    if field_value
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_whitespace())
    {
        Ok(())
    } else {
        Err(OperatorValidationError::FieldValidationError {
            field: field_name.to_string(),
            message: format!("{} must contain only letters and spaces", field_name),
            received: Some(field_value.to_string()),
        })
    }
}

pub fn format_country_name(raw_country: &str) -> Result<String, ErrorResponse> {
    let name = validate_non_empty("country", raw_country).map_err(ErrorResponse::Validation)?;
    validate_characters_and_spaces("country", name).map_err(ErrorResponse::Validation)?;
    let mut chars = name.chars();
    let formatted =
        chars.next().unwrap().to_uppercase().to_string() + &chars.as_str().to_lowercase();
    Ok(formatted)
}

pub fn validate_iso_fields(patch: &PatchOperator) -> Result<(), ErrorResponse> {
    if let Some(ref iso2) = patch.iso2 {
        if iso2.len() != 2
            || !iso2
                .chars()
                .all(|arg0: char| char::is_ascii_alphabetic(&arg0))
        {
            return Err(OperatorValidationError::FieldValidationError {
                field: "iso2".to_string(),
                message: "ISO2 must be exactly 2 letters".to_string(),
                received: Some(iso2.clone()),
            }
            .into());
        }
    }
    if let Some(ref iso3) = patch.iso3 {
        if iso3.len() != 3
            || !iso3
                .chars()
                .all(|arg0: char| char::is_ascii_alphabetic(&arg0))
        {
            return Err(OperatorValidationError::FieldValidationError {
                field: "iso3".to_string(),
                message: "ISO3 must be exactly 3 letters".to_string(),
                received: Some(iso3.clone()),
            }
            .into());
        }
    }
    Ok(())
}

pub fn determine_updated_codes(
    all_operators: &[Arc<Operator>],
    patch: &PatchOperator,
    current: &Operator,
) -> Result<(String, String, String), ErrorResponse> {
    if let Some(ref iso3) = patch.iso3 {
        if let Some(op) = all_operators.iter().find(|o| &o.iso3 == iso3) {
            return Ok((op.country.clone(), op.iso2.clone(), op.iso3.clone()));
        }
        return Err(ErrorResponse::NotFound {
            field: "iso3".to_string(),
            received: iso3.clone(),
            expected: "an existing ISO3 code".to_string(),
        });
    }
    if let Some(ref iso2) = patch.iso2 {
        if let Some(op) = all_operators.iter().find(|o| &o.iso2 == iso2) {
            return Ok((op.country.clone(), op.iso2.clone(), op.iso3.clone()));
        }
        return Err(ErrorResponse::NotFound {
            field: "iso2".to_string(),
            received: iso2.clone(),
            expected: "an existing ISO2 code".to_string(),
        });
    }
    if let Some(ref country_raw) = patch.country {
        let formatted = format_country_name(country_raw)?;
        if let Some(op) = all_operators.iter().find(|o| o.country == formatted) {
            return Ok((op.country.clone(), op.iso2.clone(), op.iso3.clone()));
        }
        return Err(ErrorResponse::NotFound {
            field: "country".to_string(),
            received: formatted.clone(),
            expected: "an existing country".to_string(),
        });
    }
    Ok((
        current.country.clone(),
        current.iso2.clone(),
        current.iso3.clone(),
    ))
}

pub fn validate_patch_fields(
    dup: &OperatorDuplicateChecker<'_>,
    patch: &PatchOperator,
) -> Result<(), ErrorResponse> {
    if let Some(list) = patch.e212.as_deref() {
        for code in list {
            validate_digits(SubscriberIdKind::Imsi, "e212", code)?;
            if dup.imsi_prefixes.contains(code.as_str()) {
                return Err(OperatorValidationError::DuplicateImsiError {
                    field: "e212".to_string(),
                    received: code.clone(),
                }
                .into());
            }
        }
    }

    if let Some(list) = patch.e164.as_deref() {
        for code in list {
            validate_digits(SubscriberIdKind::Msisdn, "e164", code)?;
            if dup.msisdn_prefixes.contains(code.as_str()) {
                return Err(OperatorValidationError::DuplicateMsisdnError {
                    field: "e164".to_string(),
                    received: code.clone(),
                }
                .into());
            }
        }
    }

    if let Some(Some(name)) = patch.name.as_ref() {
        let key = name.trim().to_lowercase();
        if dup.operator_names.contains(key.as_str()) {
            return Err(OperatorValidationError::FieldValidationError {
                field: "name".to_string(),
                message: format!("Operator name '{}' already exists", name),
                received: Some(name.clone()),
            }
            .into());
        }
    }

    if let Some(list) = patch.tadig.as_deref() {
        for code in list {
            if !TADIG_RE.is_match(code) {
                return Err(OperatorValidationError::InvalidTadigError {
                    field: "tadig".to_string(),
                    received: code.clone(),
                    expected: "5–9 chars, letters/digits/_ only".to_string(),
                }
                .into());
            }
            if dup.tadig_codes.contains(code.as_str()) {
                return Err(OperatorValidationError::DuplicateTadigError {
                    field: "tadig".to_string(),
                    received: code.clone(),
                }
                .into());
            }
        }
    }

    Ok(())
}

pub fn find_operator_by_country<'a>(
    list: &'a [Operator],
    country: &str,
) -> Result<&'a Operator, ErrorResponse> {
    list.iter().find(|op| op.country == country).ok_or_else(|| {
        ErrorResponse::Validation(OperatorValidationError::InvalidCountry {
            field: "country".to_string(),
            received: country.to_string(),
            expected: "an existing country".to_string(),
        })
    })
}

pub fn validate_unique_numeric_codes<B>(
    field: &str,
    _code_type: &str,
    codes: Option<&[String]>,
    existing: &HashSet<B>,
) -> Result<(), ErrorResponse>
where
    B: Borrow<str> + Display + Eq + Hash,
{
    if let Some(list) = codes {
        for code in list {
            if !code.chars().all(|c| c.is_ascii_digit()) {
                let err = if field == "e212" {
                    OperatorValidationError::InvalidImsiError {
                        field: field.to_string(),
                        received: code.clone(),
                        expected: "digits only".to_string(),
                    }
                } else {
                    OperatorValidationError::InvalidMsisdnError {
                        field: field.to_string(),
                        received: code.clone(),
                        expected: "digits only".to_string(),
                    }
                };
                return Err(err.into());
            }
            if existing.contains(code) {
                let dup = if field == "e212" {
                    OperatorValidationError::DuplicateImsiError {
                        field: field.to_string(),
                        received: code.clone(),
                    }
                } else {
                    OperatorValidationError::DuplicateMsisdnError {
                        field: field.to_string(),
                        received: code.clone(),
                    }
                };
                return Err(dup.into());
            }
        }
    }
    Ok(())
}

pub fn validate_unique_imsi_codes<B>(
    imsi_codes: Option<&[String]>,
    existing_imsi_prefixes: &HashSet<B>,
) -> Result<(), ErrorResponse>
where
    B: Borrow<str> + Display + Eq + Hash,
{
    validate_unique_numeric_codes("e212", "IMSI prefix", imsi_codes, existing_imsi_prefixes)
}

pub fn validate_unique_msisdn_codes<B>(
    msisdn_codes: Option<&[String]>,
    existing_msisdn_prefixes: &HashSet<B>,
) -> Result<(), ErrorResponse>
where
    B: Borrow<str> + Display + Eq + Hash,
{
    validate_unique_numeric_codes(
        "e164",
        "MSISDN prefix",
        msisdn_codes,
        existing_msisdn_prefixes,
    )
}

pub fn validate_unique_operator_name<B>(
    operator_name: &Option<String>,
    existing_operator_names: &HashSet<B>,
) -> Result<(), ErrorResponse>
where
    B: Borrow<str> + Eq + Hash,
{
    if let Some(name) = operator_name {
        let key = name.trim().to_lowercase();
        if existing_operator_names.contains(key.as_str()) {
            return Err(OperatorValidationError::FieldValidationError {
                field: "name".to_string(),
                message: format!("Operator name '{}' already exists", name),
                received: Some(name.clone()),
            }
            .into());
        }
    }
    Ok(())
}

pub fn validate_iso3_code(field_name: &str, iso3: &str) -> Result<String, OperatorValidationError> {
    let uppercase_iso3 = iso3.to_ascii_uppercase();

    let is_valid_iso3 = uppercase_iso3.len() == 3
        && uppercase_iso3
            .chars()
            .all(|chars| chars.is_ascii_uppercase());

    if is_valid_iso3 {
        Ok(uppercase_iso3)
    } else {
        Err(OperatorValidationError::FieldValidationError {
            field: field_name.to_string(),
            message: "ISO3 must be exactly 3 ASCII letters".to_string(),
            received: Some(iso3.to_string()),
        })
    }
}

pub fn validate_unique_tadig_codes<B>(
    tadig_codes: &[String],
    existing_tadig_codes: &HashSet<B>,
) -> Result<(), ErrorResponse>
where
    B: Borrow<str> + Display + Eq + Hash,
{
    for code in tadig_codes {
        let valid_chars = code.chars().all(|c| c == '_' || c.is_ascii_alphanumeric());
        if !(5..=9).contains(&code.len()) || !valid_chars {
            return Err(OperatorValidationError::InvalidTadigError {
                field: "tadig".to_string(),
                received: code.clone(),
                expected: "5–9 chars, letters/digits/_ only".to_string(),
            }
            .into());
        }
        if existing_tadig_codes.contains(code) {
            return Err(OperatorValidationError::DuplicateTadigError {
                field: "tadig".to_string(),
                received: code.clone(),
            }
            .into());
        }
    }
    Ok(())
}

pub fn format_network_name(network_name_input: &str) -> String {
    let name = network_name_input.trim();
    if name.is_empty() {
        return String::new();
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    let rest = chars.as_str().to_lowercase();

    first + &rest
}
