use super::db;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptySubscription, FieldResult, InputObject, Object, Schema, SimpleObject, ID,
};
use sqlx::PgPool as Pool;
use tide::{
    http::{headers, mime},
    Body, Error, Request, Response, StatusCode,
};
use uuid::Uuid;

/// A todo
#[derive(SimpleObject)]
pub struct Todo {
    /// The id of the todo
    id: ID,
    /// The title of the todo
    title: String,
    /// Is the todo completed?
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

#[derive(InputObject)]
struct NewTodo {
    title: String,
}

#[derive(InputObject)]
struct UpdateTodo {
    title: Option<String>,
    completed: Option<bool>,
}

struct AuthSubject(String);

#[derive(Clone)]
pub struct State {
    pub schema: Schema<QueryRoot, MutationRoot, EmptySubscription>,
    pub mounted_at: String,
}

impl State {
    pub fn new(pool: Pool, mounted_at: &str) -> State {
        State {
            schema: Schema::build(QueryRoot, MutationRoot, EmptySubscription)
                .data(pool)
                .finish(),
            mounted_at: mounted_at.into(),
        }
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get all Todos
    async fn todos(&self, context: &Context<'_>) -> FieldResult<Vec<Todo>> {
        let pool = context.data()?;
        let auth_subject = context.data::<AuthSubject>()?;
        let todos = db::Todo::find_all(pool, &auth_subject.0).await?;
        Ok(todos.iter().cloned().map(Into::into).collect())
    }

    /// Get Todo by id
    async fn todo(&self, context: &Context<'_>, id: ID) -> FieldResult<Todo> {
        let pool = context.data()?;
        let auth_subject = context.data::<AuthSubject>()?;
        let id = Uuid::parse_str(id.as_str())?;
        let todo = db::Todo::find_by_id(pool, &auth_subject.0, id).await?;
        Ok(todo.into())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new todo (returns the created todo)
    async fn create_todo(&self, context: &Context<'_>, todo: NewTodo) -> FieldResult<Todo> {
        let pool = context.data()?;
        let auth_subject = context.data::<AuthSubject>()?;
        let todo = db::Todo::create(pool, &auth_subject.0, todo.title).await?;
        Ok(todo.into())
    }

    /// Update a todo (returns the updated todo)
    async fn update_todo(
        &self,
        context: &Context<'_>,
        id: ID,
        todo: UpdateTodo,
    ) -> FieldResult<Todo> {
        let pool = context.data()?;
        let auth_subject = context.data::<AuthSubject>()?;
        let todo = db::Todo::update(
            pool,
            &auth_subject.0,
            Uuid::parse_str(id.as_str())?,
            todo.title,
            todo.completed,
        )
        .await?;
        Ok(todo.into())
    }

    /// Delete a todo (returns the deleted todo)
    async fn delete_todo(&self, context: &Context<'_>, id: ID) -> FieldResult<Todo> {
        let pool = context.data()?;
        let auth_subject = context.data::<AuthSubject>()?;
        let todo = db::Todo::delete(pool, &auth_subject.0, Uuid::parse_str(id.as_str())?).await?;
        Ok(todo.into())
    }
}

pub async fn handle_graphql(req: Request<State>) -> tide::Result {
    let header = req
        .header(&headers::AUTHORIZATION)
        .map(|header| header.last().to_string())
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "missing Authorization header"))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "missing Bearer token"))?;

    let claims = super::jwt::decode_jwt(token).map_err(|e| {
        Error::from_str(
            StatusCode::Unauthorized,
            format!("cannot decode token {:?}", e),
        )
    })?;
    let auth = AuthSubject(claims.sub.clone());

    let schema = req.state().schema.clone();

    let req = async_graphql_tide::receive_request(req).await?.data(auth);

    async_graphql_tide::respond(schema.execute(req).await)
}

pub async fn handle_graphiql(req: Request<State>) -> tide::Result {
    let mounted_at = req.state().mounted_at.clone();
    let mut response = Response::new(StatusCode::Ok);
    response.set_body(Body::from_string(playground_source(
        GraphQLPlaygroundConfig::new(&mounted_at),
    )));
    response.set_content_type(mime::HTML);

    Ok(response)
}
