-- Your SQL goes here
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email VARCHAR NOT NULL UNIQUE,
    display_name VARCHAR NOT NULL,
    password VARCHAR NOT NULL
)