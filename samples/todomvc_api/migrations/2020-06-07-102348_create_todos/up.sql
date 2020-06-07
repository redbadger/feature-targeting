CREATE TABLE todos
(
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT 'f',
    item_order INTEGER NULL
);
