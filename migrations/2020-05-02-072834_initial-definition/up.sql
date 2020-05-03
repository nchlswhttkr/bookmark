-- Your SQL goes here
CREATE TABLE bookmark (
    id INTEGER NOT NULL PRIMARY KEY,
    "url" text UNIQUE NOT NULL,
    "name" text NOT NULL DEFAULT '',
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);