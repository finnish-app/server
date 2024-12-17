BEGIN;
ALTER TABLE expenses
    ADD COLUMN bank_source text,
    ADD COLUMN external_account_id uuid,
    ADD COLUMN external_id uuid UNIQUE,
    ADD COLUMN external_created_at timestamptz;
ALTER TABLE expenses
    ALTER COLUMN category DROP NOT NULL;
ALTER TABLE pluggy_items
    ADD COLUMN last_updated_at timestamptz NOT NULL;
COMMIT;
