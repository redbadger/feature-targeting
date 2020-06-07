use super::schema::todos;

#[derive(Queryable)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
    #[column_name = "item_order"]
    pub order: Option<i32>,
}

#[derive(Insertable)]
#[table_name = "todos"]
pub struct NewTodo<'a> {
    pub title: &'a str,
    #[column_name = "item_order"]
    pub order: Option<i32>,
}
