use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Operator {
    pub country: String,
    #[serde(default)]
    pub e164: Option<Vec<String>>,
    #[serde(default)]
    pub e212: Option<Vec<String>>,
    pub iso2: String,
    pub iso3: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub realm: Option<Vec<String>>,
    #[serde(default)]
    pub tadig: Option<Vec<String>>,
}

impl Operator {
    pub fn has_tadig(&self, key: &str) -> bool {
        self.tadig
            .as_ref()
            .is_some_and(|codes| codes.iter().any(|c| c == key))
    }

    pub fn from_create(country: String, iso2: String, iso3: String, input: CreateOperator) -> Self {
        let tadig_list = input.tadig.unwrap_or_default();
        Operator {
            country,
            e164: input.e164,
            e212: input.e212,
            iso2,
            iso3,
            name: input.name,
            realm: input.realm,
            tadig: Some(tadig_list),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOperator {
    pub country: String,
    pub e164: Option<Vec<String>>,
    pub e212: Option<Vec<String>>,
    pub name: Option<String>,
    pub realm: Option<Vec<String>>,
    pub tadig: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PatchOperator {
    pub country: Option<String>,
    pub iso2: Option<String>,
    pub iso3: Option<String>,
    pub e164: Option<Vec<String>>,
    pub e212: Option<Vec<String>>,
    pub name: Option<Option<String>>,
    pub realm: Option<Vec<String>>,
    pub tadig: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct GroupOperatorsSummary {
    pub iso3: String,
    pub total: usize,
    pub operators: Vec<Operator>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadQuery {
    pub imsi: Option<String>,
    pub msisdn: Option<String>,
    pub tadig: Option<String>,
    pub iso3: Option<String>,
}

#[derive(Debug, Clone)]
pub enum QueryType {
    Imsi,
    Msisdn,
    Tadig,
    Iso3,
}

#[derive(Debug, Clone)]
pub enum OperatorValidationError {
    InvalidImsiError {
        field: String,
        received: String,
        expected: String,
    },
    DuplicateImsiError {
        field: String,
        received: String,
    },
    InvalidMsisdnError {
        field: String,
        received: String,
        expected: String,
    },
    DuplicateMsisdnError {
        field: String,
        received: String,
    },
    InvalidTadigError {
        field: String,
        received: String,
        expected: String,
    },
    DuplicateTadigError {
        field: String,
        received: String,
    },
    InvalidCountry {
        field: String,
        received: String,
        expected: String,
    },
    FieldValidationError {
        field: String,
        message: String,
        received: Option<String>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum SubscriberIdKind {
    Imsi,
    Msisdn,
}

#[derive(Serialize, Debug)]
pub struct RoamingPartnersResult {
    pub message: String,
    pub bordering_countries: Vec<String>,
    pub partners: Vec<Operator>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct OperatorSizeGroups {
    pub small_size_operators: usize,
    pub medium_size_operators: usize,
    pub large_size_operators: usize,

    pub small_operators: Vec<Operator>,
    pub medium_operators: Vec<Operator>,
    pub large_operators: Vec<Operator>,
}

impl SubscriberIdKind {
    pub fn error_kind(&self, field: String, received: String) -> OperatorValidationError {
        match self {
            SubscriberIdKind::Imsi => OperatorValidationError::InvalidImsiError {
                field,
                received,
                expected: "digits only".into(),
            },
            SubscriberIdKind::Msisdn => OperatorValidationError::InvalidMsisdnError {
                field,
                received,
                expected: "digits only".into(),
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OperatorDuplicateChecker<'a> {
    pub imsi_prefixes: HashSet<&'a str>,
    pub msisdn_prefixes: HashSet<&'a str>,
    pub operator_names: HashSet<&'a str>,
    pub tadig_codes: HashSet<&'a str>,
}

impl<'a> OperatorDuplicateChecker<'a> {
    pub fn from_operators(ops: &'a [Operator]) -> Self {
        let imsi_prefixes = ops
            .iter()
            .filter_map(|op| op.e212.as_ref())
            .flat_map(|v| v.iter().map(String::as_str))
            .collect();

        let msisdn_prefixes = ops
            .iter()
            .filter_map(|op| op.e164.as_ref())
            .flat_map(|v| v.iter().map(String::as_str))
            .collect();

        let operator_names = ops
            .iter()
            .filter_map(|op| op.name.as_ref())
            .map(|n| {
                let lowered = n.trim().to_lowercase();
                Box::leak(lowered.into_boxed_str()) as &str
            })
            .collect();

        let tadig_codes = ops
            .iter()
            .filter_map(|op| op.tadig.as_ref())
            .flat_map(|v| v.iter().map(String::as_str))
            .collect();

        OperatorDuplicateChecker {
            imsi_prefixes,
            msisdn_prefixes,
            operator_names,
            tadig_codes,
        }
    }

    pub fn exclude(&mut self, op: &Operator) {
        for code in op.e212.iter().flatten() {
            self.imsi_prefixes.remove(code.as_str());
        }
        for code in op.e164.iter().flatten() {
            self.msisdn_prefixes.remove(code.as_str());
        }
        if let Some(name) = &op.name {
            let key = name.trim().to_lowercase();
            self.operator_names.remove(key.as_str());
        }
        for code in op.tadig.iter().flatten() {
            self.tadig_codes.remove(code.as_str());
        }
    }
}

