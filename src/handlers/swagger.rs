use actix_web::web;
use crate::utils::config::configure_swagger as internal_configure;

pub fn configure_swagger(config: &mut web::ServiceConfig) {
    internal_configure(config);
}
