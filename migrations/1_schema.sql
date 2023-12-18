DROP TABLE IF EXISTS expenses;

CREATE TABLE IF NOT EXISTS expenses (
    id serial PRIMARY KEY,
    amount real NOT NULL
);
