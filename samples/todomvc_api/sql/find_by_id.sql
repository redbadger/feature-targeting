SELECT id,
    title,
    completed
FROM todos
WHERE id = $1
