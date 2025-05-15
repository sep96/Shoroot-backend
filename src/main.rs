mod config;
mod db;
mod auth;
mod bets;
mod users;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on http://127.0.0.1:8080");
    // TODO: Load config, connect db, mount routes
    Ok(())
}