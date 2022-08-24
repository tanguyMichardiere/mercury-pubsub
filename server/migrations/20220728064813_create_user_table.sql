CREATE TABLE "User" (
    id               uuid           PRIMARY KEY DEFAULT gen_random_uuid(),
    name             varchar(16)    UNIQUE NOT NULL CHECK (length(name) >= 4),
    password_hash    char(34)       NOT NULL,
    rank             int            NOT NULL CHECK (rank >= 0)
);

INSERT INTO "User" (name, password_hash, rank)
    VALUES ('admin', crypt('mercury', gen_salt('md5')), 0);
