INSERT INTO todos (title, item_order)
VALUES ($1, $2)
RETURNING id,
    title,
    completed,
    item_order AS order
