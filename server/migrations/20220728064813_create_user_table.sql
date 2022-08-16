CREATE TABLE "User" (
    id               uuid           PRIMARY KEY DEFAULT gen_random_uuid(),
    name             varchar(16)    UNIQUE NOT NULL,
    password_hash    char(34)       NOT NULL
);
