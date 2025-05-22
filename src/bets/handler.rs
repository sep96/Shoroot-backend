use actix_web::{post, web, HttpResponse, Responder};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::user::{PlaceBetInput, Bet};
use crate::auth::jwt::Claims;
use actix_web::HttpRequest;
use serde_json::json;

fn extract_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    let auth_header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = auth_header.trim_start_matches("Bearer ").to_string();
    let jwt_secret = std::env::var("JWT_SECRET").ok()?;
    let claims = Claims::decode_token(&token, &jwt_secret).ok()?;
    Uuid::parse_str(&claims.sub).ok()
}

#[post("/api/bets")]
pub async fn place_bet(
    req: HttpRequest,
    db: web::Data<PgPool>,
    input: web::Json<PlaceBetInput>,
) -> impl Responder {
    let user_id = match extract_user_id_from_request(&req) {
        Some(uid) => uid,
        None => return HttpResponse::Unauthorized().json(json!({"error": "Invalid token"})),
    };

    let bet_id = Uuid::new_v4();

    let result = sqlx::query!(
        "INSERT INTO bets (id, user_id, event_id, predicted_winner, amount) VALUES ($1, $2, $3, $4, $5)",
        bet_id,
        user_id,
        input.event_id,
        input.predicted_winner,
        input.amount
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Bet placed successfully"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Failed to place bet"})),
    }
}
