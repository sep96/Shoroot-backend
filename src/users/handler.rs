use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::user::{RegisterInput, LoginInput, User};
use crate::auth::jwt::create_jwt;
use argon2::{self, Config};

#[post("/api/register")]
pub async fn register_user(
    db: web::Data<PgPool>,
    input: web::Json<RegisterInput>,
) -> impl Responder {
    let password_hash = hash_password(&input.password);
    let user_id = Uuid::new_v4();

    let result = sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
        user_id,
        input.username,
        input.email,
        password_hash
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "User registered successfully"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Registration failed"})),
    }
}

#[post("/api/login")]
pub async fn login_user(
    db: web::Data<PgPool>,
    input: web::Json<LoginInput>,
) -> impl Responder {
    let result = sqlx::query_as!(
        User,
        "SELECT id, username, email, password_hash, balance FROM users WHERE username = $1",
        input.username
    )
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(user) if verify_password(&input.password, &user.password_hash) => {
            let token = create_jwt(user.id.to_string());
            HttpResponse::Ok().json(json!({ "token": token }))
        },
        _ => HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"})),
    }
}

fn hash_password(password: &str) -> String {
    let salt = b"somesalt";
    argon2::hash_encoded(password.as_bytes(), salt, &Config::default()).unwrap()
}

fn verify_password(password: &str, hash: &str) -> bool {
    argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false)
}
