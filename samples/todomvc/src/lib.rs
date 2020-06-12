use enclose::enc;
use graphql_client::{GraphQLQuery, Response};
use indexmap::IndexMap;
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};
use std::mem;
use uuid::Uuid;
use web_sys::HtmlInputElement;

const ENTER_KEY: u32 = 13;
const ESC_KEY: u32 = 27;
const API_URL: &str = "http://localhost:3030/graphql";

type TodoId = Uuid;

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
generate_query!(GetAll);
generate_query!(DeleteTodo);
generate_query!(CreateTodo);
generate_query!(UpdateTodo);

async fn send_graphql_request<V, T>(variables: &V) -> fetch::Result<T>
where
    V: Serialize,
    T: for<'de> Deserialize<'de> + 'static,
{
    Request::new(API_URL)
        .method(Method::Post)
        .json(variables)?
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .subscribe(Msg::UrlChanged)
        .notify(subs::UrlChanged(url));

    orders.perform_cmd(async {
        let request = GetAll::build_query(get_all::Variables);
        let response = send_graphql_request(&request).await;
        Msg::TodosFetched(response)
    });

    Model {
        data: Data::default(),
        refs: Refs::default(),
    }
}

struct Model {
    data: Data,
    refs: Refs,
}

#[derive(Default, Serialize, Deserialize)]
struct Data {
    todos: IndexMap<TodoId, Todo>,
    filter: TodoFilter,
    new_todo_title: String,
    editing_todo: Option<EditingTodo>,
}

#[derive(Default)]
struct Refs {
    editing_todo_input: ElRef<HtmlInputElement>,
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
            Self::Active => "active",
            Self::Completed => "completed",
        }
    }
}

enum Msg {
    TodosFetched(fetch::Result<Response<get_all::ResponseData>>),
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
    CancelTodoEdit,
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
        TodosFetched(error) => log!(error),

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
                    let vars = delete_todo::Variables { id: id.to_string() };
                    orders.perform_cmd(async {
                        let request = DeleteTodo::build_query(vars);
                        let response = send_graphql_request(&request).await;
                        TodoRemoved(response)
                    });
                }
            }
        }

        CreateNewTodo => {
            if !data.new_todo_title.is_empty() {
                let vars = create_todo::Variables {
                    title: mem::take(&mut data.new_todo_title),
                };
                orders.perform_cmd(async {
                    let request = CreateTodo::build_query(vars);
                    let response = send_graphql_request(&request).await;
                    NewTodoCreated(response)
                });
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
        NewTodoCreated(error) => log!(error),

        ToggleTodo(todo_id) => {
            if let Some(todo) = data.todos.get(&todo_id) {
                let vars = update_todo::Variables {
                    id: todo_id.to_string(),
                    title: Some(todo.title.clone()),
                    completed: Some(!todo.completed),
                };
                orders.perform_cmd(async {
                    let request = UpdateTodo::build_query(vars);
                    let response = send_graphql_request(&request).await;
                    TodoToggled(response)
                });
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
        TodoToggled(error) => log!(error),
        ToggleAll => {
            let target_state = !data.todos.values().all(|todo| todo.completed);

            for (id, todo) in &data.todos {
                if todo.completed != target_state {
                    let cmd = ToggleTodo(*id);
                    orders.perform_cmd(async { cmd });
                }
            }
        }

        RemoveTodo(todo_id) => {
            let id = todo_id.to_string();
            orders.perform_cmd(async {
                let request = DeleteTodo::build_query(delete_todo::Variables { id });
                let response = send_graphql_request(&request).await;
                TodoRemoved(response)
            });
        }
        TodoRemoved(Ok(Response {
            data: Some(response_data),
            ..
        })) => match Uuid::parse_str(response_data.delete_todo.id.as_str()) {
            Ok(deleted_id) => {
                data.todos.shift_remove(&deleted_id);
            }
            Err(e) => {
                log!("Failed to parse id of deleted todo as Uuid::V4 ({:?})", e);
            }
        },
        TodoRemoved(error) => log!(error),

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
            if let Some(editing_todo) = data.editing_todo.take() {
                if let Some(todo) = data.todos.get_mut(&editing_todo.id) {
                    todo.title = editing_todo.title;
                }
            }
        }
        CancelTodoEdit => {
            data.editing_todo = None;
        }
    }
}

fn view(model: &Model) -> impl IntoNodes<Msg> {
    let data = &model.data;
    nodes![
        view_header(&data.new_todo_title),
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

fn view_header(new_todo_title: &str) -> Node<Msg> {
    header![
        C!["header"],
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
    todos: &IndexMap<TodoId, Todo>,
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
    todos: &IndexMap<TodoId, Todo>,
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
                ev(
                    Ev::Change,
                    enc!((todo_id) move |_| Msg::ToggleTodo(todo_id))
                )
            ],
            label![
                ev(
                    Ev::DblClick,
                    enc!((todo_id) move |_| Msg::StartTodoEdit(todo_id))
                ),
                &todo.title
            ],
            button![
                C!["destroy"],
                ev(Ev::Click, enc!((todo_id) move |_| Msg::RemoveTodo(todo_id)))
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

fn view_footer(todos: &IndexMap<TodoId, Todo>, filter: TodoFilter) -> Node<Msg> {
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

fn view_filters(filter: TodoFilter) -> Node<Msg> {
    ul![
        C!["filters"],
        view_filter("All", TodoFilter::All, filter),
        view_filter("Active", TodoFilter::Active, filter),
        view_filter("Completed", TodoFilter::Completed, filter),
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

fn view_clear_completed(todos: &IndexMap<TodoId, Todo>) -> Option<Node<Msg>> {
    let completed_count = todos.values().filter(|todo| todo.completed).count();

    IF!(completed_count > 0 => {
        button![
            C!["clear-completed"],
            ev(Ev::Click, |_| Msg::ClearCompletedTodos),
            format!("Clear completed ({})", completed_count),
        ]
    })
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
