BEGIN;

ALTER TYPE expense_category RENAME TO expense_category_old;

CREATE TYPE expense_category AS ENUM (
    'restaurants',
    'shopping',
    'services',
    'entertainment',
    'groceries',
    'salary',
    'interest Income',
    'utilities',
    'pharmacy',
    'transfer',
    'transport',
    'others'
);

ALTER TABLE expenses
    ALTER COLUMN category TYPE expense_category
    USING category::text::expense_category;

DROP TYPE expense_category_old;

COMMIT;
