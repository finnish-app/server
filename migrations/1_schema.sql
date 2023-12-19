DROP TABLE IF EXISTS expenses;

DO $$ BEGIN
CREATE TYPE expense_type AS ENUM ('food', 'transport', 'health', 'education', 'entertainment', 'other');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

CREATE TABLE IF NOT EXISTS expenses (
    id serial PRIMARY KEY,
    description varchar(255) NOT NULL,
    price real NOT NULL,
    expense_type expense_type NOT NULL,
    is_essencial boolean NOT NULL
);
