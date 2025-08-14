use crate::app_state::AppState;
use crate::operators::crud_operations::read::lookup_operator_by_query;
use crate::utils::error_responses::ErrorResponse;
use crate::utils::models::{Operator, QueryType, ReadQuery};
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use serde_json;
use std::sync::Arc;

pub async fn handle_get_operator(
    app_data: web::Data<AppState>,
    query: web::Query<ReadQuery>,
) -> Result<HttpResponse, ErrorResponse> {
    let operators = app_data
        .operators
        .read()
        .map_err(|_| ErrorResponse::InternalError)?;

    if query.imsi.is_none() && query.msisdn.is_none() && query.tadig.is_none() {
        let all_ops: Vec<&Operator> = operators.iter().map(|op| op.as_ref()).collect();
        return Ok(HttpResponse::Ok().json(all_ops));
    }

    let (search_str, search_type) = if let Some(ref imsi) = query.imsi {
        (imsi.as_str(), QueryType::Imsi)
    } else if let Some(ref msisdn) = query.msisdn {
        (msisdn.as_str(), QueryType::Msisdn)
    } else {
        (query.tadig.as_ref().unwrap().as_str(), QueryType::Tadig)
    };

    let maybe_op = lookup_operator_by_query(&app_data, search_str, search_type)?;

    match maybe_op {
        None => Ok(HttpResponse::NotFound().finish()),
        Some(op_arc) => {
            let body = serde_json::to_string_pretty(Arc::as_ref(&op_arc))
                .map_err(|_| ErrorResponse::InternalError)?;
            Ok(HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body))
        }
    }
}
