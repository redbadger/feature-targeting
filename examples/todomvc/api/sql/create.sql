INSERT INTO todos (auth_subject, title)
VALUES ($1, $2)
RETURNING id,
    auth_subject,
    title,
    completed
