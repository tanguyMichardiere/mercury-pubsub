CREATE TABLE "User" (
    id               uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    name             text    UNIQUE NOT NULL,
    password_hash    text    NOT NULL
);
