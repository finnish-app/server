BEGIN;
-- ALTER TABLE pluggy_items
--     DROP COLUMN last_updated_at;
ALTER TABLE expenses
    ALTER COLUMN category SET NOT NULL;
ALTER TABLE expenses
    DROP COLUMN bank_source,
    DROP COLUMN external_account_id,
    DROP COLUMN external_id,
    DROP COLUMN external_created_at;
COMMIT;
