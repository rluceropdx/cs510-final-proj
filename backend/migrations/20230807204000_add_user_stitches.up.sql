-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_stitches
(
    id  serial PRIMARY KEY,
    user_id    int NOT NULL,
    search_terms    VARCHAR(255) NOT NULL,
    image_data      BYTEA,
    created_on TIMESTAMPTZ    NOT NULL DEFAULT NOW()
)