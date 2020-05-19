-- Your SQL goes here
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email VARCHAR NOT NULL,
    display_name VARCHAR NOT NULL,
    password_hash VARCHAR NOT NULL
)