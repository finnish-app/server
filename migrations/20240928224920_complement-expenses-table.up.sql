ALTER TABLE expenses
    ALTER COLUMN date DROP DEFAULT,
    ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE expenses
    ADD COLUMN created_at timestamptz,
    ADD COLUMN deleted_at timestamptz;

ALTER TABLE expenses
    ADD CONSTRAINT unique_uuid UNIQUE (uuid);

UPDATE
    expenses
SET
    created_at = now();

ALTER TABLE expenses
    ALTER COLUMN created_at SET NOT NULL;
