use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, dev::ServerHandle, web};
use dotenv::dotenv;
use std::io;

use operator_mappings_api::{
    app_state::{
        load_operator_mappings, loaders::load_country_borders, persistence::spawn_persistence_tasks,
    },
    handlers::swagger::configure_swagger,
    utils::{config::load, routes::configure_routes},
};

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut app_state = load_operator_mappings()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let borders = load_country_borders()
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    app_state.borders = borders;

    let shared_state = web::Data::new(app_state);
    let server_state = shared_state.clone();
    let spawn_state = shared_state.clone();

    let cfg = load();
    let bind_addr = format!("{}:{}", cfg.host, cfg.port);
    println!("Server running at http://{}", bind_addr);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(server_state.clone())
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Found()
                        .append_header(("Location", "/swagger-ui/"))
                        .finish()
                }),
            )
            .configure(configure_routes)
            .configure(configure_swagger)
    })
    .workers(cfg.workers)
    .bind(&bind_addr)?
    .run();

    let handle: ServerHandle = server.handle();
    spawn_persistence_tasks(spawn_state, cfg.save_interval_minutes, handle);

    server.await
}
