INSERT INTO todos (title)
VALUES ($1)
RETURNING id,
    title,
    completed
