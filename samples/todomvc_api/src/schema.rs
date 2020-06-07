table! {
    todos (id) {
        id -> Int4,
        title -> Varchar,
        completed -> Bool,
        item_order -> Nullable<Int4>,
    }
}
