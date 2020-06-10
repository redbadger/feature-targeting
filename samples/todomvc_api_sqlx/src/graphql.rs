use super::db;
use anyhow::Result;
use juniper::{EmptySubscription, FieldError, RootNode};
use sqlx::{PgConnection as Connection, Pool};
use tide::{Body, Request, Response, StatusCode};

#[derive(Clone)]
pub struct Todo {
    id: Option<u32>,
    title: String,
    completed: bool,
    order: Option<i32>,
}

#[juniper::graphql_object]
#[graphql(description = "A todo")]
impl Todo {
    #[graphql(description = "A todo id")]
    fn id(&self) -> i32 {
        self.id.unwrap_or(0) as i32
    }

    #[graphql(description = "The title of the todo")]
    fn title(&self) -> &str {
        &self.title
    }

    #[graphql(description = "Is the todo completed?")]
    fn completed(&self) -> bool {
        self.completed
    }

    #[graphql(description = "The order of the todo")]
    fn order(&self) -> Option<i32> {
        self.order
    }
}

impl From<db::Todo> for Todo {
    fn from(d: db::Todo) -> Self {
        Self {
            id: Some(d.id as u32),
            title: d.title,
            completed: d.completed,
            order: d.order,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
struct NewTodo {
    title: String,
    order: Option<i32>,
}

pub struct State {
    pub todos: Pool<Connection>,
}

impl juniper::Context for State {}

pub struct Query;

#[juniper::graphql_object(Context=State)]
impl Query {
    #[graphql(description = "Get all Todos")]
    async fn todos(context: &State) -> Result<Vec<Todo>, FieldError> {
        let todos = db::Todo::find_all(&context.todos).await?;
        Ok(todos.iter().cloned().map(Into::into).collect())
    }
}

pub struct Mutation;

#[juniper::graphql_object(Context=State)]
impl Mutation {
    #[graphql_object(description = "Add new todo")]
    async fn add_todo(context: &State, todo: NewTodo) -> Result<Todo, FieldError> {
        let todo = db::Todo::create(todo.title, todo.order, &context.todos).await?;
        Ok(todo.into())
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<State>>;
fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {}, EmptySubscription::<State>::new())
}

pub async fn handle_graphql(mut cx: Request<State>) -> tide::Result {
    lazy_static! {
        static ref SCHEMA: Schema = create_schema();
    };

    let query: juniper::http::GraphQLRequest = cx
        .body_json()
        .await
        .expect("be able to deserialize the graphql request");

    let response = query.execute(&SCHEMA, cx.state()).await;

    let status = if response.is_ok() {
        StatusCode::Ok
    } else {
        StatusCode::BadRequest
    };

    let mut res = Response::new(status);
    res.set_body(Body::from_json(&response)?);
    Ok(res)
}

pub async fn handle_graphiql(_: Request<State>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(juniper::http::graphiql::graphiql_source("/graphql", None));
    res.set_content_type(tide::http::mime::HTML);
    Ok(res)
}
