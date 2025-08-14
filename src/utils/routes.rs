use crate::handlers::{
    create::handle_create_operator,
    delete::handle_delete_operator,
    find_roaming_partners::handle_find_roaming_partners,
    get_operators_by_e212_count::handle_get_operators_by_e212_count,
    group_operators_by_iso3::handle_get_country_mapping,
    network_names::handle_network_names,
    read::handle_get_operator,
    update::{handle_update_by_patch, handle_update_by_put},
};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/operators")
            .route("", web::get().to(handle_get_operator))
            .route("", web::post().to(handle_create_operator))
            .route("/{tadig}", web::put().to(handle_update_by_put))
            .route("/{tadig}", web::patch().to(handle_update_by_patch))
            .route("/{tadig}", web::delete().to(handle_delete_operator))
            .route("/network-names", web::get().to(handle_network_names))
            .route(
                "/by-countries-operators",
                web::get().to(handle_get_country_mapping),
            )
            .route(
                "/grouped-by-e212",
                web::get().to(handle_get_operators_by_e212_count),
            )
            .route(
                "/roaming-partners",
                web::get().to(handle_find_roaming_partners),
            ),
    );
}
