DELETE FROM todos
WHERE id = $1
    AND auth_subject = $2
RETURNING id,
    auth_subject,
    title,
    completed
