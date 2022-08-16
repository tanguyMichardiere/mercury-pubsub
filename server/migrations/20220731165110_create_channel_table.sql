CREATE TABLE "Channel" (
    id        uuid            PRIMARY KEY DEFAULT gen_random_uuid(),
    name      varchar(16)     UNIQUE NOT NULL,
    schema    JSONB           NOT NULL
);
