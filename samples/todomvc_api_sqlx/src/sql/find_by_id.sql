SELECT id,
    title,
    completed,
    item_order as order
FROM todos
WHERE id = $1
