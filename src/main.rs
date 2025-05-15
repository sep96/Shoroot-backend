use actix_web::{App, HttpServer, web};
use config::Config;
use db::connect;
use auth::handler::register;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let config = Config::from_env();
    let pool = connect(&config.database_url).await;

    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
