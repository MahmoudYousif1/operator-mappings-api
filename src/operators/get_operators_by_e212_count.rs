use crate::{
    app_state::model::AppState,
    utils::{
        error_responses::ErrorResponse,
        models::{Operator, OperatorSizeGroups},
    },
};
use std::sync::{Arc, RwLockReadGuard};

const SMALL_E212_MAX: usize = 3;
const MEDIUM_E212_MAX: usize = 7;

fn classify_single_operator(groups: &mut OperatorSizeGroups, operator: Operator) {
    let e212_count = operator.e212.as_ref().map(|v| v.len()).unwrap_or(0);

    if e212_count <= SMALL_E212_MAX {
        groups.small_operators.push(operator);
        groups.small_size_operators += 1;
    } else if e212_count <= MEDIUM_E212_MAX {
        groups.medium_operators.push(operator);
        groups.medium_size_operators += 1;
    } else {
        groups.large_operators.push(operator);
        groups.large_size_operators += 1;
    }
}

fn categorize_operators_by_e212_count(operators: &[Arc<Operator>]) -> OperatorSizeGroups {
    operators
        .iter()
        .map(|arc_operator| (**arc_operator).clone())
        .fold(OperatorSizeGroups::default(), |mut groups, operator| {
            classify_single_operator(&mut groups, operator);
            groups
        })
}

pub fn get_operators_grouped_by_e212_count(
    state: &AppState,
) -> Result<OperatorSizeGroups, ErrorResponse> {
    let operators_guard: RwLockReadGuard<'_, Vec<Arc<Operator>>> = state
        .operators
        .read()
        .map_err(|_| ErrorResponse::InternalError)?;
    let groups = categorize_operators_by_e212_count(&operators_guard);

    Ok(groups)
}
