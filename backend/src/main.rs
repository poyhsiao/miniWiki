use actix_web::{web, App, HttpServer, Responder};
use dotenv::dotenv;
use std::sync::Arc;

mod routes;
mod config;

use config::Config;
use shared_database::connection::init_database;

async fn health() -> impl Responder {
    actix_web::web::Json(serde_json::json!({
        "status": "healthy",
        "service": "miniwiki-api",
        "version": "0.1.0"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::from_env().expect("Failed to load configuration");
    
    let host = config.host.clone();
    let port = config.port;

    println!("Starting miniWiki API server at http://{}:{}", host, port);

    let db = Arc::new(
        init_database(&config.database_url)
            .await
            .expect("Failed to connect to database")
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .configure(routes::config)
            .route("/health", web::get().to(health))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
