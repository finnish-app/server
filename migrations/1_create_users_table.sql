DROP TABLE IF EXISTS users;

CREATE TABLE IF NOT EXISTS users (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username text NOT NULL UNIQUE,
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    verification_code varchar(255),
    code_expires_at timestamptz,
    verified boolean NOT NULL DEFAULT false
);

CREATE INDEX idx_verification_code ON users (verification_code);
