CREATE TYPE keytype AS ENUM ('publisher', 'subscriber');

CREATE TABLE "Key" (
    id      uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    type    keytype     NOT NULL,
    hash    char(34)    NOT NULL
);

CREATE TABLE "Access" (
    key_id        uuid    REFERENCES "Key" ON DELETE CASCADE NOT NULL,
    channel_id    uuid    REFERENCES "Channel" ON DELETE CASCADE NOT NULL,

    PRIMARY KEY (key_id, channel_id)
);
