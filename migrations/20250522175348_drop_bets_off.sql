-- Add migration script here
CREATE TABLE IF NOT EXISTS bets (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    event_id UUID NOT NULL,
    predicted_winner TEXT NOT NULL,
    amount BIGINT NOT NULL,
    status TEXT DEFAULT 'pending'
);
