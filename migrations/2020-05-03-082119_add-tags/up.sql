-- Your SQL goes here
CREATE TABLE tag (
    id INTEGER NOT NULL PRIMARY KEY,
    bookmark_id INTEGER NOT NULL,
    "value" TEXT NOT NULL,
    FOREIGN KEY (bookmark_id) REFERENCES bookmark(id)
);
