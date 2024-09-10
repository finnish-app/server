CREATE TABLE pluggy_items (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id integer NOT NULL,
    external_item_id uuid NOT NULL,
    created_at timestamptz NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id)
);
