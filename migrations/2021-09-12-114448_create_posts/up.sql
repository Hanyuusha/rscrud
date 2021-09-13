-- Your SQL goes here
CREATE TABLE posts
(
    id        uuid PRIMARY KEY,
    title     VARCHAR NOT NULL,
    body      TEXT    NOT NULL,
    published BOOLEAN NOT NULL DEFAULT 'f'
)