CREATE TABLE "Session" (
    id                    uuid           PRIMARY KEY DEFAULT gen_random_uuid(),
    access_token_hash     char(34)       UNIQUE NOT NULL,
    refresh_token_hash    char(34)       UNIQUE NOT NULL,
    expires               timestamptz    NOT NULL,
    user_id               uuid           NOT NULL REFERENCES "User"
);
