use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct Todo {
    pub url: String,
    pub title: String,
    pub completed: bool,
    pub order: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct NewTodo {
    pub title: String,
    pub order: Option<i32>,
}

pub type TodoList = Vec<Todo>;
