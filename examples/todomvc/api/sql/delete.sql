DELETE FROM todos
WHERE id = $1
RETURNING id,
    title,
    completed