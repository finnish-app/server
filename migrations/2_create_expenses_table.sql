DROP TABLE IF EXISTS expenses;

DO $$ BEGIN
CREATE TYPE expense_category AS ENUM ('food', 'transport', 'health', 'education', 'entertainment', 'other');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS expenses (
    id serial PRIMARY KEY,
    description varchar(255) NOT NULL,
    price real NOT NULL,
    category expense_category NOT NULL,
    is_essential boolean NOT NULL,
    date date NOT NULL DEFAULT CURRENT_DATE,
    uuid uuid NOT NULL,
    user_id integer REFERENCES users (id) ON DELETE RESTRICT
);
