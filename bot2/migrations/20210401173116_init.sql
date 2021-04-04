CREATE TABLE IF NOT EXISTS command
(
    id      INTEGER PRIMARY KEY NOT NULL UNIQUE,
    name    TEXT                NOT NULL UNIQUE,
    code    TEXT                NOT NULL
);

CREATE TABLE IF NOT EXISTS channel
(
    id      INTEGER PRIMARY KEY NOT NULL UNIQUE,
    name    TEXT                NOT NULL UNIQUE,
    prefix  TEXT                NOT NULL DEFAULT "!",
    joined  INTEGER             NOT NULL DEFAULT 1
);