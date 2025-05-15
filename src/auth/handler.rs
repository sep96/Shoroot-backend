use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use serde::Deserialize;
use crate::auth::jwt::create_jwt;

#[derive(Deserialize)]
pub struct RegisterData {
    username: String,
    email: String,
    password: String,
}

#[post("/api/auth/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    data: web::Json<RegisterData>,
) -> impl Responder {
    let user_id = uuid::Uuid::new_v4();
    let hashed_password = data.password.clone(); // هش واقعی اضافه شود
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
        user_id,
        data.username,
        data.email,
        hashed_password
    )
    .execute(pool.get_ref())
    .await
    .unwrap();

    HttpResponse::Ok().json("User registered")
}
