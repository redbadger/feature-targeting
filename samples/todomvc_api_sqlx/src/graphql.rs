use juniper::RootNode;
use std::sync::RwLock;
use tide::{Body, Request, Response, StatusCode};

#[derive(Clone)]
pub struct Todo {
    id: Option<u16>,
    title: String,
    completed: bool,
    order: Option<i32>,
}

#[juniper::object]
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

#[derive(juniper::GraphQLInputObject)]
struct NewTodo {
    title: String,
    order: Option<i32>,
}

impl NewTodo {
    fn into_internal(self) -> Todo {
        Todo {
            id: None,
            title: self.title,
            completed: false,
            order: self.order,
        }
    }
}

pub struct State {
    pub todos: RwLock<Vec<Todo>>,
}

impl juniper::Context for State {}

pub struct QueryRoot;

#[juniper::object(Context=State)]
impl QueryRoot {
    #[graphql(description = "Get all Todos")]
    fn todos(context: &State) -> Vec<Todo> {
        let todos = context.todos.read().unwrap();
        todos.iter().cloned().collect()
    }
}

pub struct MutationRoot;

#[juniper::object(Context=State)]
impl MutationRoot {
    #[graphql(description = "Add new todo")]
    fn add_todo(context: &State, todo: NewTodo) -> Todo {
        let mut todos = context.todos.write().unwrap();
        let mut todo = todo.into_internal();
        todo.id = Some((todos.len() + 1) as u16);
        todos.push(todo.clone());
        todo
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;
fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}

pub async fn handle_graphql(mut cx: Request<State>) -> tide::Result {
    let query: juniper::http::GraphQLRequest = cx
        .body_json()
        .await
        .expect("be able to deserialize the graphql request");

    let schema = create_schema(); // probably worth making the schema a singleton using lazy_static library
    let response = query.execute(&schema, cx.state());
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
    res.set_body(juniper::http::graphiql::graphiql_source("/graphql"));
    res.set_content_type(tide::http::mime::HTML);
    Ok(res)
}
