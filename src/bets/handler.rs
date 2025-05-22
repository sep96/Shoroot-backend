use actix_web::{post, get, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;
use sqlx::PgPool;
use serde::{Deserialize};
use serde_json::json;

use crate::auth::jwt::Claims;
use crate::models::user::{PlaceBetInput, Bet, BetForSettlement};

fn extract_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    let auth_header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = auth_header.trim_start_matches("Bearer ").to_string();
    let jwt_secret = std::env::var("JWT_SECRET").ok()?;
    let claims = Claims::decode_token(&token, &jwt_secret).ok()?;
    Uuid::parse_str(&claims.sub).ok()
}

#[derive(Debug, Deserialize)]
pub struct EventResultInput {
    pub winner: String,
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

#[get("/api/bets")]
pub async fn get_user_bets(
    req: HttpRequest,
    db: web::Data<PgPool>,
) -> impl Responder {
    let user_id = match extract_user_id_from_request(&req) {
        Some(uid) => uid,
        None => return HttpResponse::Unauthorized().json(json!({"error": "Invalid token"})),
    };

    let bets = sqlx::query_as::<_, Bet>(
        "SELECT id, user_id, event_id, predicted_winner, amount, status FROM bets WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_all(db.get_ref())
    .await;

    match bets {
        Ok(bets) => HttpResponse::Ok().json(bets),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Failed to fetch bets"})),
    }
}

#[post("/api/admin/events/{event_id}/result")]
pub async fn submit_event_result(
    path: web::Path<Uuid>,
    db: web::Data<PgPool>,
    input: web::Json<EventResultInput>,
) -> impl Responder {
    let event_id = path.into_inner();

    let update_result = sqlx::query!(
        "UPDATE events SET result = $1 WHERE id = $2",
        input.winner,
        event_id
    )
    .execute(db.get_ref())
    .await;

    if update_result.is_err() {
        return HttpResponse::InternalServerError().json(json!({"error": "Failed to update event result"}));
    }

    let bets = sqlx::query_as::<_, BetForSettlement>(
        "SELECT id, user_id, predicted_winner, amount FROM bets WHERE event_id = $1 AND status = 'pending'"
    )
    .bind(event_id)
    .fetch_all(db.get_ref())
    .await;

    if let Ok(bets) = bets {
        for bet in bets {
            let is_win = bet.predicted_winner == input.winner;

            if is_win {
                let _ = sqlx::query!(
                    "UPDATE users SET balance = balance + $1 WHERE id = $2",
                    bet.amount * 2,
                    bet.user_id
                )
                .execute(db.get_ref())
                .await;

                let _ = sqlx::query!(
                    "UPDATE bets SET status = 'won' WHERE id = $1",
                    bet.id
                )
                .execute(db.get_ref())
                .await;
            } else {
                let _ = sqlx::query!(
                    "UPDATE bets SET status = 'lost' WHERE id = $1",
                    bet.id
                )
                .execute(db.get_ref())
                .await;
            }
        }

        HttpResponse::Ok().json(json!({"message": "Event result processed and bets settled"}))
    } else {
        HttpResponse::InternalServerError().json(json!({"error": "Failed to load related bets"}))
    }
}
