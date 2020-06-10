UPDATE todos
SET title = COALESCE($1, title),
    completed = COALESCE($2, completed),
    item_order = COALESCE($3, item_order)
WHERE id = $4
RETURNING id,
    title,
    completed,
    item_order AS order
