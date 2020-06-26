UPDATE todos
SET title = COALESCE($1, title),
    completed = COALESCE($2, completed)
WHERE id = $3
RETURNING id,
    title,
    completed
