-- This file should undo anything in `up.sql`

ALTER TABLE tag RENAME TO tag_old;

CREATE TABLE tag (
    id INTEGER NOT NULL PRIMARY KEY,
    bookmark_id INTEGER NOT NULL,
    "value" TEXT NOT NULL,
    FOREIGN KEY (bookmark_id) REFERENCES bookmark(id)
);

INSERT INTO tag SELECT * FROM tag_old;

DROP TABLE tag_old;