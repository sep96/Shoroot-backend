use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub balance: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Bet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub predicted_winner: String,
    pub amount: i64,
    pub status: String, // "pending", "won", "lost"
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PlaceBetInput {
    pub event_id: Uuid,
    pub predicted_winner: String,
    pub amount: i64,
}

#[derive(Debug, sqlx::FromRow, ToSchema)]
pub struct BetForSettlement {
    pub id: Uuid,
    pub user_id: Uuid,
    pub predicted_winner: String,
    pub amount: i64,
}