use crate::utils::models::Operator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type CountryBordersMap = HashMap<String, Vec<String>>;

pub struct AppState {
    pub operators: RwLock<Vec<Arc<Operator>>>,
    pub mappings_file_path: String,
    pub borders: CountryBordersMap,
}

impl AppState {
    pub fn new(
        initial_operators: Vec<Operator>,
        mappings_file_path: String,
        borders_map: CountryBordersMap,
    ) -> Self {
        let operators = initial_operators.into_iter().map(Arc::new).collect();
        AppState {
            operators: RwLock::new(operators),
            mappings_file_path,
            borders: borders_map,
        }
    }
}
