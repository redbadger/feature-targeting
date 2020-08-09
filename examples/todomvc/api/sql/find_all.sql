SELECT id,
    auth_subject,
    title,
    completed
FROM todos
WHERE auth_subject = $1
ORDER BY id
