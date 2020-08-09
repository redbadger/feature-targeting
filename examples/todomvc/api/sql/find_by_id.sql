SELECT id,
    auth_subject,
    title,
    completed
FROM todos
WHERE id = $1
    AND auth_subject = $2
