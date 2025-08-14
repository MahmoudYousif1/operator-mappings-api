use crate::app_state::model::{AppState, CountryBordersMap};
use crate::utils::error_responses::ErrorResponse;
use crate::utils::models::{Operator, RoamingPartnersResult};

fn find_base_country_by_tadig(
    operators: &[Operator],
    requested_tadig: &str,
) -> Result<String, ErrorResponse> {
    operators
        .iter()
        .find(|op| {
            op.tadig
                .as_ref()
                .map(|tads| tads.iter().any(|t| t.eq_ignore_ascii_case(requested_tadig)))
                .unwrap_or(false)
        })
        .map(|op| op.country.clone())
        .ok_or_else(|| ErrorResponse::NotFound {
            field: "tadig".to_string(),
            received: requested_tadig.to_string(),
            expected: "an existing TADIG".to_string(),
        })
}

fn lookup_neighbor_names(
    borders_map: &CountryBordersMap,
    base_country: &str,
) -> Result<Vec<String>, ErrorResponse> {
    if let Some(neighbors) = borders_map.get(base_country) {
        return Ok(neighbors.clone());
    }
    let lc_base = base_country.to_lowercase();
    for (csv_country, neighbors) in borders_map.iter() {
        if lc_base.contains(&csv_country.to_lowercase()) {
            return Ok(neighbors.clone());
        }
    }
    Err(ErrorResponse::BorderCountryNotFound {
        field: "country".to_string(),
        received: base_country.to_string(),
        expected: "a country with known borders".to_string(),
    })
}

fn collect_partners_by_country(
    operators: &[Operator],
    neighbor_names_lc: &[String],
) -> Vec<Operator> {
    operators
        .iter()
        .filter_map(|op| {
            let cand_country_lc = op.country.to_lowercase();
            if neighbor_names_lc
                .iter()
                .any(|nbr| cand_country_lc.contains(nbr))
            {
                Some(op.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn find_roaming_partners(
    state: &AppState,
    requested_tadig: &str,
) -> Result<RoamingPartnersResult, ErrorResponse> {
    let base_country: String = {
        let operators_read_guard = state
            .operators
            .read()
            .map_err(|_| ErrorResponse::InternalError)?;
        let operators_list: Vec<Operator> = operators_read_guard
            .iter()
            .map(|arc| (**arc).clone())
            .collect();
        find_base_country_by_tadig(&operators_list, requested_tadig)?
    };
    let neighbor_names = lookup_neighbor_names(&state.borders, &base_country)?;
    let neighbor_names_lc: Vec<String> = neighbor_names.iter().map(|n| n.to_lowercase()).collect();
    let partners: Vec<Operator> = {
        let operators_read_guard = state
            .operators
            .read()
            .map_err(|_| ErrorResponse::InternalError)?;
        let operators_list: Vec<Operator> = operators_read_guard
            .iter()
            .map(|arc| (**arc).clone())
            .collect();
        collect_partners_by_country(&operators_list, &neighbor_names_lc)
    };
    let message = format!("Found {} roaming partners", partners.len());
    Ok(RoamingPartnersResult {
        message,
        bordering_countries: neighbor_names,
        partners,
    })
}
