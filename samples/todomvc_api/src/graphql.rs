use super::db;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, FieldResult, InputObject, Object, Schema, SimpleObject, ID,
};
use sqlx::{PgConnection as Connection, Pool};
use tide::{http::mime, Body, Request, Response, StatusCode};
use uuid::Uuid;

#[SimpleObject(desc = "A todo")]
pub struct Todo {
    #[field(desc = "The id of the todo")]
    id: ID,
    #[field(desc = "The title of the todo")]
    title: String,
    #[field(desc = "Is the todo completed?")]
    completed: bool,
}

impl From<db::Todo> for Todo {
    fn from(d: db::Todo) -> Self {
        Self {
            id: d.id.into(),
            title: d.title,
            completed: d.completed,
        }
    }
}

#[InputObject]
struct NewTodo {
    title: String,
}

#[InputObject]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

pub struct State {
    pub schema: Schema<QueryRoot, MutationRoot, EmptySubscription>,
}

impl State {
    pub fn new(pool: Pool<Connection>) -> State {
        State {
            schema: Schema::build(QueryRoot, MutationRoot, EmptySubscription)
                .data(pool)
                .finish(),
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    #[field(desc = "Get all Todos")]
    async fn todos(&self, context: &Context<'_>) -> FieldResult<Vec<Todo>> {
        let todos = db::Todo::find_all(&context.data()).await?;
        Ok(todos.iter().cloned().map(Into::into).collect())
    }

    #[field(desc = "Get Todo by id")]
    async fn todo(&self, context: &Context<'_>, id: ID) -> FieldResult<Todo> {
        let id = Uuid::parse_str(id.as_str())?;
        let todo = db::Todo::find_by_id(id, &context.data()).await?;
        Ok(todo.into())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    #[field(desc = "Create a new todo (returns the created todo)")]
    async fn create_todo(&self, context: &Context<'_>, todo: NewTodo) -> FieldResult<Todo> {
        let todo = db::Todo::create(todo.title, &context.data()).await?;
        Ok(todo.into())
    }

    #[field(desc = "Update a todo (returns the updated todo)")]
    async fn update_todo(
        &self,
        context: &Context<'_>,
        id: ID,
        todo: UpdateTodo,
    ) -> FieldResult<Todo> {
        let todo = db::Todo::update(
            Uuid::parse_str(id.as_str())?,
            todo.title,
            todo.completed,
            &context.data(),
        )
        .await?;
        Ok(todo.into())
    }

    #[field(desc = "Delete a todo (returns the deleted todo)")]
    async fn delete_todo(&self, context: &Context<'_>, id: ID) -> FieldResult<Todo> {
        let todo = db::Todo::delete(Uuid::parse_str(id.as_str())?, &context.data()).await?;
        Ok(todo.into())
    }
}

pub async fn handle_graphql(req: Request<State>) -> tide::Result {
    let schema = req.state().schema.clone();
    async_graphql_tide::graphql(req, schema, |query_builder| query_builder).await
}

pub async fn handle_graphiql(_: Request<State>) -> tide::Result {
    let mut response = Response::new(StatusCode::Ok);
    response.set_body(Body::from_string(playground_source(
        GraphQLPlaygroundConfig::new("/graphql"),
    )));
    response.set_content_type(mime::HTML);

    Ok(response)
}
