DROP TABLE IF EXISTS users;

CREATE TABLE IF NOT EXISTS users (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username text NOT NULL UNIQUE,
    password text NOT NULL
);

INSERT INTO users (username, password)
VALUES ('admin', '$argon2id$v=19$m=19456,t=2,p=1$VE0e3g7DalWHgDwou3nuRA$uC6TER156UQpk0lNQ5+jHM0l5poVjPA1he/Tyn9J4Zw');
