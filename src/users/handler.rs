use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use sqlx::Row;
use crate::models::user::{RegisterInput, LoginInput, User};
use crate::auth::jwt::{create_jwt, Claims};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, PasswordHash, rand_core::OsRng}
};

fn extract_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    let auth_header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = auth_header.trim_start_matches("Bearer ").to_string();
    let jwt_secret = std::env::var("JWT_SECRET").ok()?;
    let claims = Claims::decode_token(&token, &jwt_secret).ok()?;
    Uuid::parse_str(&claims.sub).ok()
}

#[get("/api/balance")]
pub async fn get_user_balance(
    req: HttpRequest,
    db: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match extract_user_id_from_request(&req) {
        Some(uid) => uid,
        None => return HttpResponse::Unauthorized().json(json!({"error": "Invalid token"})),
    };

    let result = sqlx::query("SELECT balance FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(db.get_ref())
        .await;

    match result {
        Ok(row) => {
            let balance: i64 = row.try_get("balance").unwrap_or(0);
            HttpResponse::Ok().json(json!({ "balance": balance }))
        },
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Failed to fetch balance"})),
    }
}

#[post("/api/register")]
pub async fn register_user(
    db: web::Data<PgPool>,
    input: web::Json<RegisterInput>,
) -> impl Responder {
    let password_hash = hash_password(&input.password);
    let user_id = Uuid::new_v4();

    let result = sqlx::query(
        "INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)"
    )
    .bind(user_id)
    .bind(&input.username)
    .bind(&input.email)
    .bind(password_hash)
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
    let result = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, balance FROM users WHERE username = $1"
    )
    .bind(&input.username)
    .fetch_one(db.get_ref())
    .await;

    match result {
        Ok(user) if verify_password(&input.password, &user.password_hash) => {
            let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
            let token = create_jwt(&user.id.to_string(), &jwt_secret);
            HttpResponse::Ok().json(json!({ "token": token }))
        },
        _ => HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"})),
    }
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn verify_password(password: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
