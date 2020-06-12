CREATE TABLE IF NOT EXISTS todos (
    id uuid DEFAULT uuid_generate_v4 (),
    title TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    item_order INTEGER NULL,
    PRIMARY KEY (id)
);
