UPDATE todos
SET auth_subject = $2,
    title = COALESCE($3, title),
    completed = COALESCE($4, completed)
WHERE id = $1
RETURNING id,
    auth_subject,
    title,
    completed
