UPDATE todos
SET title = $1,
    completed = $2,
    item_order = $3
WHERE id = $4
RETURNING id,
    title,
    completed,
    item_order AS order
