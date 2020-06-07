#[macro_use]
extern crate diesel;

pub mod api;
pub mod models;
pub mod schema;

use anyhow::Result;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use models::*;
use std::{env, sync::Arc};
use tide::{Body, Request, Response, Server};

pub struct State {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

pub fn db_pool() -> Result<Arc<Pool<ConnectionManager<PgConnection>>>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    Ok(Arc::new(Pool::builder().max_size(4).build(manager)?))
}

pub fn create_app() -> Result<Server<State>> {
    let mut app = tide::with_state(State { pool: db_pool()? });
    app.at("/").get(get_todos_handler);
    app.at("/").post(create_todos_handler);
    Ok(app)
}

async fn get_todos_handler(req: Request<State>) -> tide::Result {
    use schema::todos::dsl::*;
    let connection = req.state().pool.get()?;
    let list: api::TodoList = todos
        .limit(100)
        .load::<Todo>(&connection)?
        .iter()
        .map(Into::into)
        .collect();
    let mut res = Response::new(200);
    res.set_body(Body::from_json(&list)?);
    Ok(res)
}

async fn create_todos_handler(mut req: Request<State>) -> tide::Result {
    use schema::todos;

    let new_todo: api::NewTodo = req.body_json().await?;
    let new_todo = NewTodo {
        title: new_todo.title.as_str(),
        order: new_todo.order,
    };

    let connection = req.state().pool.get()?;

    let t: Todo = diesel::insert_into(todos::table)
        .values(&new_todo)
        .get_result(&connection)?;
    let t: api::Todo = (&t).into();

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&t)?);
    Ok(res)
}

impl From<&Todo> for api::Todo {
    fn from(t: &Todo) -> Self {
        Self {
            url: format!("{}", t.id),
            title: t.title.clone(),
            completed: t.completed,
            order: t.order,
        }
    }
}
