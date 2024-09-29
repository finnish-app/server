ALTER TABLE expenses
    DROP CONSTRAINT unique_uuid;

ALTER TABLE expenses
    DROP COLUMN deleted_at,
    DROP COLUMN created_at;

ALTER TABLE expenses
    ALTER COLUMN user_id DROP NOT NULL,
    ALTER COLUMN date SET DEFAULT CURRENT_DATE;
