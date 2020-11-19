use graphql_client::{GraphQLQuery, Response};
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};
use std::mem;
use uuid::Uuid;
use web_sys::HtmlInputElement;

mod cookies;
mod session;

const ENTER_KEY: u32 = 13;
const ESC_KEY: u32 = 27;
const DEFAULT_API_URL: &str = "http://localhost:3030";
const ACTIVE: &str = "active";
const COMPLETED: &str = "completed";

type TodoId = Uuid;
type Store = indexmap::IndexMap<TodoId, Todo>;

macro_rules! generate_query {
    ($query:ident) => {
        #[derive(GraphQLQuery)]
        #[graphql(
            schema_path = "graphql/schema.graphql",
            query_path = "graphql/queries.graphql",
            response_derives = "Debug"
        )]
        struct $query;
    };
}
generate_query!(GetTodos);
generate_query!(DeleteTodo);
generate_query!(CreateTodo);
generate_query!(UpdateTodo);

async fn send_graphql_request<V, T>(
    api_url: &url::Url,
    jwt: &str,
    variables: &V,
) -> fetch::Result<T>
where
    V: Serialize,
    T: for<'de> Deserialize<'de> + 'static,
{
    Request::new(api_url.to_string())
        .method(Method::Post)
        .header(Header::authorization(format!("Bearer {}", jwt)))
        .json(variables)?
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

pub struct Model {
    api_url: url::Url,
    data: Data,
    refs: Refs,
    session: session::Model,
}

#[derive(Default, Serialize, Deserialize)]
struct Data {
    todos: Store,
    filter: TodoFilter,
    new_todo_title: String,
    editing_todo: Option<EditingTodo>,
}

#[derive(Default)]
struct Refs {
    editing_todo_input: ElRef<HtmlInputElement>,
}

struct_urls!();
impl<'a> Urls<'a> {
    pub fn home(self) -> Url {
        self.base_url()
    }
    pub fn active(self) -> Url {
        self.base_url().add_hash_path_part(ACTIVE)
    }
    pub fn completed(self) -> Url {
        self.base_url().add_hash_path_part(COMPLETED)
    }
}

#[derive(Serialize, Deserialize)]
struct Todo {
    title: String,
    completed: bool,
}

#[derive(Serialize, Deserialize)]
struct EditingTodo {
    id: TodoId,
    title: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TodoFilter {
    All,
    Active,
    Completed,
}

impl Default for TodoFilter {
    fn default() -> Self {
        Self::All
    }
}

impl TodoFilter {
    fn to_url_path(self) -> &'static str {
        match self {
            Self::All => "",
            Self::Active => ACTIVE,
            Self::Completed => COMPLETED,
        }
    }
}

pub enum Msg {
    TodosFetched(fetch::Result<Response<get_todos::ResponseData>>),
    UrlChanged(subs::UrlChanged),

    NewTodoTitleChanged(String),

    ClearCompletedTodos,

    CreateNewTodo,
    NewTodoCreated(fetch::Result<Response<create_todo::ResponseData>>),

    ToggleTodo(TodoId),
    TodoToggled(fetch::Result<Response<update_todo::ResponseData>>),
    ToggleAll,

    RemoveTodo(TodoId),
    TodoRemoved(fetch::Result<Response<delete_todo::ResponseData>>),

    StartTodoEdit(TodoId),
    EditingTodoTitleChanged(String),
    SaveEditingTodo,
    EditingTodoSaved(fetch::Result<Response<update_todo::ResponseData>>),
    CancelTodoEdit,

    Session(session::Msg),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    let data = &mut model.data;
    use Msg::*;
    match msg {
        TodosFetched(Ok(Response {
            data: Some(response_data),
            ..
        })) => {
            for todo in response_data.todos.iter() {
                data.todos.insert(
                    Uuid::parse_str(todo.id.as_str()).expect("Cannot parse todo.id as Uuid::V4"),
                    Todo {
                        title: todo.title.clone(),
                        completed: todo.completed,
                    },
                );
            }
        }
        TodosFetched(error) => error!(error),

        UrlChanged(subs::UrlChanged(mut url)) => {
            data.filter = match url.next_path_part() {
                Some(path_part) if path_part == TodoFilter::Active.to_url_path() => {
                    TodoFilter::Active
                }
                Some(path_part) if path_part == TodoFilter::Completed.to_url_path() => {
                    TodoFilter::Completed
                }
                _ => TodoFilter::All,
            };
        }
        NewTodoTitleChanged(title) => {
            data.new_todo_title = title;
        }

        ClearCompletedTodos => {
            for (id, todo) in &data.todos {
                if todo.completed {
                    if let Some(jwt) = model.session.jwt.clone() {
                        let vars = delete_todo::Variables { id: id.to_string() };
                        let api_url = model.api_url.clone();
                        orders.skip().perform_cmd(async move {
                            let request = DeleteTodo::build_query(vars);
                            let response = send_graphql_request(&api_url, &jwt, &request).await;
                            TodoRemoved(response)
                        });
                    };
                }
            }
        }

        CreateNewTodo => {
            if !data.new_todo_title.is_empty() {
                if let Some(jwt) = model.session.jwt.clone() {
                    let vars = create_todo::Variables {
                        title: mem::take(&mut data.new_todo_title),
                    };
                    let api_url = model.api_url.clone();
                    orders.skip().perform_cmd(async move {
                        let request = CreateTodo::build_query(vars);
                        let response = send_graphql_request(&api_url, &jwt, &request).await;
                        NewTodoCreated(response)
                    });
                };
            }
        }
        NewTodoCreated(Ok(Response {
            data: Some(response_data),
            ..
        })) => {
            let todo = response_data.create_todo;
            data.todos.insert(
                Uuid::parse_str(&todo.id).expect("Failed to parse id of deleted todo as Uuid::V4"),
                Todo {
                    title: todo.title,
                    completed: todo.completed,
                },
            );
        }
        NewTodoCreated(error) => error!(error),

        ToggleTodo(todo_id) => {
            if let Some(todo) = data.todos.get(&todo_id) {
                if let Some(jwt) = model.session.jwt.clone() {
                    let vars = update_todo::Variables {
                        id: todo_id.to_string(),
                        title: Some(todo.title.clone()),
                        completed: Some(!todo.completed),
                    };
                    let api_url = model.api_url.clone();
                    orders.skip().perform_cmd(async move {
                        let request = UpdateTodo::build_query(vars);
                        let response = send_graphql_request(&api_url, &jwt, &request).await;
                        TodoToggled(response)
                    });
                };
            }
        }
        TodoToggled(Ok(Response {
            data: Some(response_data),
            ..
        })) => {
            let id = Uuid::parse_str(&response_data.update_todo.id)
                .expect("Failed to parse id of deleted todo as Uuid::V4");
            if let Some(todo) = data.todos.get_mut(&id) {
                todo.completed = !todo.completed;
            }
        }
        TodoToggled(error) => error!(error),
        ToggleAll => {
            let target_state = !data.todos.values().all(|todo| todo.completed);

            for (id, todo) in &data.todos {
                if todo.completed != target_state {
                    let cmd = ToggleTodo(*id);
                    orders.skip().perform_cmd(async { cmd });
                }
            }
        }

        RemoveTodo(todo_id) => {
            if let Some(jwt) = model.session.jwt.clone() {
                let id = todo_id.to_string();
                let api_url = model.api_url.clone();
                orders.skip().perform_cmd(async move {
                    let request = DeleteTodo::build_query(delete_todo::Variables { id });
                    let response = send_graphql_request(&api_url, &jwt, &request).await;
                    TodoRemoved(response)
                });
            };
        }
        TodoRemoved(Ok(Response {
            data: Some(response_data),
            ..
        })) => match Uuid::parse_str(response_data.delete_todo.id.as_str()) {
            Ok(deleted_id) => {
                data.todos.shift_remove(&deleted_id);
            }
            Err(e) => error!("Failed to parse id of deleted todo as Uuid::V4 ({:?})", e),
        },
        TodoRemoved(error) => error!(error),

        StartTodoEdit(todo_id) => {
            if let Some(todo) = data.todos.get(&todo_id) {
                data.editing_todo = Some({
                    EditingTodo {
                        id: todo_id,
                        title: todo.title.clone(),
                    }
                });
            }

            let input = model.refs.editing_todo_input.clone();
            orders.after_next_render(move |_| {
                input.get().expect("get `editing_todo_input`").select();
            });
        }
        EditingTodoTitleChanged(title) => {
            if let Some(ref mut editing_todo) = data.editing_todo {
                editing_todo.title = title
            }
        }
        SaveEditingTodo => {
            if let Some(todo) = data.editing_todo.take() {
                if let Some(jwt) = model.session.jwt.clone() {
                    let vars = update_todo::Variables {
                        id: todo.id.to_string(),
                        title: Some(todo.title),
                        completed: None,
                    };
                    let api_url = model.api_url.clone();
                    orders.skip().perform_cmd(async move {
                        let request = UpdateTodo::build_query(vars);
                        let response = send_graphql_request(&api_url, &jwt, &request).await;
                        EditingTodoSaved(response)
                    });
                };
            }
        }
        EditingTodoSaved(Ok(Response {
            data: Some(response_data),
            ..
        })) => match Uuid::parse_str(response_data.update_todo.id.as_str()) {
            Ok(updated_id) => {
                if let Some(todo) = data.todos.get_mut(&updated_id) {
                    todo.title = response_data.update_todo.title;
                }
            }
            Err(e) => error!("Failed to parse id of updated todo as Uuid::V4 ({:?})", e),
        },
        EditingTodoSaved(error) => error!(error),
        CancelTodoEdit => {
            data.editing_todo = None;
        }

        Session(msg) => session::update(msg, &mut model.session),
    }
}

fn view(model: &Model) -> impl IntoNodes<Msg> {
    let data = &model.data;
    nodes![
        view_header(&data.new_todo_title, &model),
        if data.todos.is_empty() {
            vec![]
        } else {
            vec![
                view_main(
                    &data.todos,
                    data.filter,
                    &data.editing_todo,
                    &model.refs.editing_todo_input,
                ),
                view_footer(&data.todos, data.filter),
            ]
        },
    ]
}

fn view_header(new_todo_title: &str, model: &Model) -> Node<Msg> {
    header![
        C!["header"],
        session::view(&model.session, Msg::Session),
        h1!["todos"],
        input![
            C!["new-todo"],
            attrs! {
                At::Placeholder => "What needs to be done?";
                At::AutoFocus => true.as_at_value();
                At::Value => new_todo_title;
            },
            keyboard_ev(Ev::KeyDown, |keyboard_event| {
                IF!(keyboard_event.key_code() == ENTER_KEY => Msg::CreateNewTodo)
            }),
            input_ev(Ev::Input, Msg::NewTodoTitleChanged),
        ]
    ]
}

fn view_main(
    todos: &Store,
    filter: TodoFilter,
    editing_todo: &Option<EditingTodo>,
    editing_todo_input: &ElRef<HtmlInputElement>,
) -> Node<Msg> {
    let all_todos_completed = todos.values().all(|todo| todo.completed);

    section![
        C!["main"],
        input![
            id!("toggle-all"),
            C!["toggle-all"],
            attrs! {
                At::Type => "checkbox",
                At::Checked => all_todos_completed.as_at_value(),
            },
            ev(Ev::Click, |_| Msg::ToggleAll)
        ],
        label![attrs! {At::For => "toggle-all"}, "Mark all as complete"],
        view_todos(todos, filter, editing_todo, editing_todo_input)
    ]
}

fn view_todos(
    todos: &Store,
    filter: TodoFilter,
    editing_todo: &Option<EditingTodo>,
    editing_todo_input: &ElRef<HtmlInputElement>,
) -> Node<Msg> {
    ul![
        C!["todo-list"],
        todos.iter().filter_map(|(todo_id, todo)| {
            let show_todo = match filter {
                TodoFilter::All => true,
                TodoFilter::Active => !todo.completed,
                TodoFilter::Completed => todo.completed,
            };
            IF!(show_todo => view_todo(todo_id, todo, editing_todo, editing_todo_input))
        })
    ]
}

#[allow(clippy::cognitive_complexity)]
fn view_todo(
    todo_id: &TodoId,
    todo: &Todo,
    editing_todo: &Option<EditingTodo>,
    editing_todo_input: &ElRef<HtmlInputElement>,
) -> Node<Msg> {
    li![
        C![
            IF!(todo.completed => "completed"),
            IF!(matches!(editing_todo, Some(editing_todo) if &editing_todo.id == todo_id) => "editing"),
        ],
        div![
            C!["view"],
            input![
                C!["toggle"],
                attrs! {
                   At::Type => "checkbox",
                   At::Checked => todo.completed.as_at_value()
                },
                ev(Ev::Change, {
                    let id = *todo_id;
                    move |_| Msg::ToggleTodo(id)
                })
            ],
            label![
                ev(Ev::DblClick, {
                    let id = *todo_id;
                    move |_| Msg::StartTodoEdit(id)
                }),
                &todo.title
            ],
            button![
                C!["destroy"],
                ev(Ev::Click, {
                    let id = *todo_id;
                    move |_| Msg::RemoveTodo(id)
                })
            ]
        ],
        match editing_todo {
            Some(editing_todo) if &editing_todo.id == todo_id => {
                input![
                    el_ref(editing_todo_input),
                    C!["edit"],
                    attrs! {At::Value => editing_todo.title},
                    ev(Ev::Blur, |_| Msg::SaveEditingTodo),
                    input_ev(Ev::Input, Msg::EditingTodoTitleChanged),
                    keyboard_ev(Ev::KeyDown, |keyboard_event| {
                        match keyboard_event.key_code() {
                            ENTER_KEY => Some(Msg::SaveEditingTodo),
                            ESC_KEY => Some(Msg::CancelTodoEdit),
                            _ => None,
                        }
                    }),
                ]
            }
            _ => empty![],
        }
    ]
}

fn view_footer(todos: &Store, filter: TodoFilter) -> Node<Msg> {
    let active_count = todos.values().filter(|todo| !todo.completed).count();

    footer![
        C!["footer"],
        span![
            C!["todo-count"],
            strong![active_count.to_string()],
            span![format!(
                " item{} left",
                if active_count == 1 { "" } else { "s" }
            )]
        ],
        view_filters(filter),
        view_clear_completed(todos),
    ]
}

fn view_filters(current_filter: TodoFilter) -> Node<Msg> {
    ul![
        C!["filters"],
        view_filter("All", TodoFilter::All, current_filter),
        view_filter("Active", TodoFilter::Active, current_filter),
        view_filter("Completed", TodoFilter::Completed, current_filter),
    ]
}

fn view_filter(title: &str, filter: TodoFilter, current_filter: TodoFilter) -> Node<Msg> {
    li![a![
        C![IF!(filter == current_filter => "selected")],
        attrs! {
            At::Href => format!("/{}", filter.to_url_path())
        },
        style! {St::Cursor => "pointer"},
        title
    ]]
}

fn view_clear_completed(todos: &Store) -> Option<Node<Msg>> {
    let completed_count = todos.values().filter(|todo| todo.completed).count();

    IF!(completed_count > 0 => {
        button![
            C!["clear-completed"],
            ev(Ev::Click, |_| Msg::ClearCompletedTodos),
            format!("Clear completed ({})", completed_count),
        ]
    })
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    session::init(orders, Msg::Session);

    let model = Model {
        api_url: url::Url::parse(&cookies::get_cookie_or_default("api_url", DEFAULT_API_URL))
            .expect("Cannot parse api_url"),
        data: Data::default(),
        refs: Refs::default(),
        session: session::Model::new(&url.to_base_url(), cookies::get_cookie("token"), None),
    };

    let api_url = model.api_url.clone();
    if let Some(jwt) = model.session.jwt.clone() {
        orders.perform_cmd(async move {
            let request = GetTodos::build_query(get_todos::Variables);
            let response = send_graphql_request(&api_url, &jwt, &request).await;
            Msg::TodosFetched(response)
        });
    };
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url));

    model
}

#[wasm_bindgen(start)]
pub fn create_app() {
    App::start("app", init, update, view);
}
