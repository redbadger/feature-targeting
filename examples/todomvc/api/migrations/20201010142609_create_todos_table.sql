CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS todos (
    id uuid DEFAULT uuid_generate_v4 (),
    auth_subject VARCHAR(256) NOT NULL,
    title TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (id)
);
