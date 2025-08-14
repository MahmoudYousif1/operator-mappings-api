use operator_mappings_api::{
    app_state::AppState,
    operators::crud_operations::update::{update_operator_by_patch, update_operator_by_put},
    utils::models::{CreateOperator, Operator, PatchOperator},
};
use std::collections::HashMap;

fn initial_operators() -> Vec<Operator> {
    vec![
        Operator {
            country: "Ireland".to_string(),
            iso2: "IE".to_string(),
            iso3: "IRL".to_string(),
            name: Some("IrishCom".to_string()),
            e212: Some(vec!["27201".to_string()]),
            e164: Some(vec!["35385".to_string()]),
            realm: None,
            tadig: Some(vec!["IRL01".to_string()]),
        },
        Operator {
            country: "Romania".to_string(),
            iso2: "RO".to_string(),
            iso3: "ROU".to_string(),
            name: Some("RomNet".to_string()),
            e212: Some(vec!["22605".to_string()]),
            e164: Some(vec!["40770".to_string()]),
            realm: None,
            tadig: Some(vec!["ROM05".to_string()]),
        },
    ]
}

fn setup_state() -> AppState {
    AppState::new(
        initial_operators(),
        "test_mapping.json".to_string(),
        HashMap::new(),
    )
}

#[test]
fn test_patch_update_name_and_msisdn() {
    let state = setup_state();

    let patch = PatchOperator {
        name: Some(Some("Updated IrishCom".to_string())),
        e164: Some(vec!["35386".to_string()]),
        ..Default::default()
    };

    let result = update_operator_by_patch(&state, "IRL01", patch);
    assert!(
        result.is_ok(),
        "expected successful patch, got {:?}",
        result.err()
    );

    let ops = state.operators.read().unwrap();
    let op = ops
        .iter()
        .find(|o| o.has_tadig("IRL01"))
        .expect("operator IRL01 must still exist after patch");

    assert_eq!(op.name.as_ref().unwrap(), "Updated IrishCom");
    assert_eq!(op.e164.as_ref().unwrap(), &vec!["35386".to_string()]);
}

#[test]
fn test_patch_invalid_imsi_error() {
    let state = setup_state();

    let patch = PatchOperator {
        e212: Some(vec!["BAD12".to_string()]),
        ..Default::default()
    };

    let result = update_operator_by_patch(&state, "IRL01", patch);
    assert!(result.is_err(), "expected error for invalid IMSI");
    let err_string = result.unwrap_err().to_string();
    assert!(
        err_string.contains("InvalidImsiError"),
        "expected InvalidImsiError, got `{}`",
        err_string
    );
}

#[test]
fn test_patch_duplicate_msisdn_error() {
    let state = setup_state();

    let patch = PatchOperator {
        e164: Some(vec!["40770".to_string()]),
        ..Default::default()
    };

    let result = update_operator_by_patch(&state, "IRL01", patch);
    assert!(result.is_err(), "expected error for duplicate MSISDN");
    let err_string = result.unwrap_err().to_string();
    assert!(
        err_string.contains("DuplicateMsisdnError"),
        "expected DuplicateMsisdnError, got `{}`",
        err_string
    );
}

#[test]
fn test_patch_duplicate_tadig_error() {
    let state = setup_state();

    let patch = PatchOperator {
        tadig: Some(vec!["ROM05".to_string()]),
        ..Default::default()
    };

    let result = update_operator_by_patch(&state, "IRL01", patch);
    assert!(result.is_err(), "expected error for duplicate TADIG");
    let err_string = result.unwrap_err().to_string();
    assert!(
        err_string.contains("DuplicateTadigError"),
        "expected DuplicateTadigError, got `{}`",
        err_string
    );
}

#[test]
fn test_put_replaces_operator_successfully() {
    let state = setup_state();

    let put_data = CreateOperator {
        country: "Romania".to_string(),
        name: Some("Patched RomNet".to_string()),
        e212: Some(vec!["22699".to_string()]),
        e164: Some(vec!["40771".to_string()]),
        realm: None,
        tadig: Some(vec!["ROU99".to_string()]),
    };

    let result = update_operator_by_put(&state, "ROM05", put_data);
    assert!(
        result.is_ok(),
        "expected successful PUT, got {:?}",
        result.err()
    );

    let ops = state.operators.read().unwrap();
    assert!(
        !ops.iter().any(|o| o.has_tadig("ROM05")),
        "operator with TADIG ROM05 removed"
    );

    let new_op = ops
        .iter()
        .find(|o| o.has_tadig("ROU99"))
        .expect("new operator with tadig ROU99 must exist");

    assert_eq!(new_op.name.as_ref().unwrap(), "Patched RomNet");
    assert_eq!(new_op.e212.as_ref().unwrap(), &vec!["22699".to_string()]);
    assert_eq!(new_op.e164.as_ref().unwrap(), &vec!["40771".to_string()]);
}

#[test]
fn test_put_invalid_msisdn_error() {
    let state = setup_state();

    let bad_data = CreateOperator {
        country: "Romania".to_string(),
        name: Some("X".to_string()),
        e212: Some(vec!["22605".to_string()]),
        e164: Some(vec!["4077A".to_string()]),
        realm: None,
        tadig: Some(vec!["VALID01".to_string()]),
    };

    let result = update_operator_by_put(&state, "ROM05", bad_data);
    assert!(result.is_err(), "expected error for invalid MSISDN");
    let err_string = result.unwrap_err().to_string();
    assert!(
        err_string.contains("InvalidMsisdnError"),
        "expected InvalidMsisdnError, got `{}`",
        err_string
    );
}

#[test]
fn test_put_duplicate_imsi_error() {
    let state = setup_state();

    let dup_imsi = CreateOperator {
        country: "Ireland".to_string(),
        name: Some("Copycat".to_string()),
        e212: Some(vec!["27201".to_string()]),
        e164: Some(vec!["40100".to_string()]),
        realm: None,
        tadig: Some(vec!["NEW01".to_string()]),
    };

    let result = update_operator_by_put(&state, "ROM05", dup_imsi);
    assert!(result.is_err(), "expected error for duplicate IMSI");
    let err_string = result.unwrap_err().to_string();
    assert!(
        err_string.contains("DuplicateImsiError"),
        "expected DuplicateImsiError, got `{}`",
        err_string
    );
}

#[test]
fn test_put_duplicate_tadig_error() {
    let state = setup_state();

    let dup_tadig = CreateOperator {
        country: "Ireland".to_string(),
        name: Some("TadigClash".to_string()),
        e212: Some(vec!["50101".to_string()]),
        e164: Some(vec!["50500".to_string()]),
        realm: None,
        tadig: Some(vec!["IRL01".to_string()]),
    };

    let result = update_operator_by_put(&state, "ROM05", dup_tadig);
    assert!(result.is_err(), "expected error for duplicate TADIG");
    let err_string = result.unwrap_err().to_string();
    assert!(
        err_string.contains("DuplicateTadigError"),
        "expected DuplicateTadigError, got `{}`",
        err_string
    );
}
