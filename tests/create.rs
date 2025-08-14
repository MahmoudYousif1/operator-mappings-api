use operator_mappings_api::utils::{
    models::{OperatorDuplicateChecker, OperatorValidationError, PatchOperator, SubscriberIdKind},
    validations::{TADIG_RE, validate_digits, validate_patch_fields},
};

#[test]
fn test_valid_imsi_format() {
    let result = validate_digits(SubscriberIdKind::Imsi, "e212", "72207");
    assert!(result.is_ok());
}

#[test]
fn test_invalid_imsi_format() {
    let result = validate_digits(SubscriberIdKind::Imsi, "e212", "72A07");
    match result {
        Err(OperatorValidationError::InvalidImsiError {
            field,
            received,
            expected,
        }) => {
            assert_eq!(field, "e212");
            assert_eq!(received, "72A07");
            assert_eq!(expected, "digits only");
        }
        _ => panic!("Expected InvalidImsiError"),
    }
}

#[test]
fn test_invalid_msisdn_format() {
    let result = validate_digits(SubscriberIdKind::Msisdn, "e164", "407A70");
    match result {
        Err(OperatorValidationError::InvalidMsisdnError {
            field,
            received,
            expected,
        }) => {
            assert_eq!(field, "e164");
            assert_eq!(received, "407A70");
            assert_eq!(expected, "digits only");
        }
        _ => panic!("Expected InvalidMsisdnError"),
    }
}

#[test]
fn test_invalid_tadig_format() {
    let bad_codes = vec!["!!!", "A", "TOOLOOOOOONG", "inv@lid"];

    for code in bad_codes {
        assert!(
            !TADIG_RE.is_match(code),
            "Expected TADIG '{}' to be invalid",
            code
        );
    }

    let good_code = "ARG_01";
    assert!(TADIG_RE.is_match(good_code));
}

#[test]
fn test_duplicate_imsi_prefix() {
    let patch = PatchOperator {
        e212: Some(vec!["72207".to_string()]),
        ..Default::default()
    };

    let mut dup = OperatorDuplicateChecker::default();
    dup.imsi_prefixes.insert("72207");

    let result = validate_patch_fields(&dup, &patch);
    match result {
        Err(err) => assert!(err.to_string().contains("DuplicateImsiError")),
        _ => panic!("Expected DuplicateImsiError"),
    }
}

#[test]
fn test_duplicate_msisdn_prefix() {
    let patch = PatchOperator {
        e164: Some(vec!["40770".to_string()]),
        ..Default::default()
    };

    let mut dup = OperatorDuplicateChecker::default();
    dup.msisdn_prefixes.insert("40770");

    let result = validate_patch_fields(&dup, &patch);
    match result {
        Err(err) => assert!(err.to_string().contains("DuplicateMsisdnError")),
        _ => panic!("Expected DuplicateMsisdnError"),
    }
}

#[test]
fn test_duplicate_tadig_code() {
    let patch = PatchOperator {
        tadig: Some(vec!["ARGTM".to_string()]),
        ..Default::default()
    };

    let mut dup = OperatorDuplicateChecker::default();
    dup.tadig_codes.insert("ARGTM");

    let result = validate_patch_fields(&dup, &patch);
    match result {
        Err(err) => assert!(err.to_string().contains("DuplicateTadigError")),
        _ => panic!("Expected DuplicateTadigError"),
    }
}
