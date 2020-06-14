-- Your SQL goes here
CREATE TABLE shares (
    link VARCHAR PRIMARY KEY NOT NULL,
    path VARCHAR NOT NULL
);

CREATE INDEX shared_links ON shares (link);