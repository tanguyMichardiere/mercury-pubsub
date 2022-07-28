CREATE TABLE "Session" (
    id               uuid           PRIMARY KEY DEFAULT gen_random_uuid(),
    session_token    char(64)       UNIQUE NOT NULL DEFAULT encode(gen_random_bytes(48), 'base64'),
    expires          timestamptz    NOT NULL,
    user_id          uuid           NOT NULL REFERENCES "User"
);
