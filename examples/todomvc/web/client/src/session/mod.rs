use anyhow::{anyhow, Result};
use auth::{AuthResponse, Claims};
use seed::{prelude::*, *};

mod auth;

const STORAGE_KEY: &str = "todomvc-session";

pub enum Msg {
    Login,
    LoggedIn(Option<Claims>),
    Logout,
}

pub struct Model {
    pub base_url: Url,
    pub redirect_url: url::Url,
    pub user: Option<String>,
}

impl Model {
    pub fn new(base_url: Url, redirect_url: url::Url, user: Option<String>) -> Self {
        Self {
            base_url,
            redirect_url,
            user,
        }
    }
}

pub fn update<M: 'static>(msg: Msg, model: &mut Model, orders: &mut impl Orders<M>) {
    use Msg::*;
    match msg {
        Login => {
            if let Ok(url) = auth::get_login_url(&model.redirect_url) {
                window()
                    .open_with_url_and_target(url.to_string().as_str(), "_self")
                    .expect("couldn't open window");
            }
            orders.skip();
        }
        LoggedIn(Some(claims)) => {
            model.user = Some(claims.name);
            Url::new()
                .set_path(model.base_url.hash_path())
                .go_and_replace();
        }
        LoggedIn(None) => {
            model.user = None;
        }

        Logout => {
            if let Ok(()) = auth::logout() {
                LocalStorage::remove(STORAGE_KEY).expect("problem removing key from local storage");
                model.user = None;
            }
        }
    }
}

pub fn view<M: 'static>(
    user: &Option<String>,
    to_msg: impl FnOnce(Msg) -> M + Clone + 'static,
) -> Node<M> {
    nav![
        C!["auth-text"],
        if let Some(user) = user {
            nodes![
                span![format!("{} ", user)],
                a![
                    C!["auth-link"],
                    mouse_ev(Ev::Click, |_| to_msg(Msg::Logout)),
                    "logout"
                ]
            ]
        } else {
            nodes![
                span!["Please "],
                a![
                    C!["auth-link"],
                    attrs! {
                        At::Href => "http://todo.red-badger.com/login"
                    },
                    "login"
                ],
                span![" to modify todos"]
            ]
        },
    ]
}

pub fn get_claims(url: &Url) -> Result<Option<Claims>> {
    if let Ok(response) = LocalStorage::get(STORAGE_KEY) {
        let claims = auth::get_claims(&response)?;
        return Ok(Some(claims));
    } else {
        let url = url.to_string();
        let url = url
            .strip_prefix("/#")
            .ok_or_else(|| anyhow!("url doesn't start with \"/#\""))?;
        if let Ok(response) = serde_urlencoded::from_str::<AuthResponse>(url) {
            let claims = auth::get_claims(&response)?;
            LocalStorage::insert(STORAGE_KEY, &response)
                .map_err(|e| anyhow!("cannot insert into localstorage {:?}", e))?;
            return Ok(Some(claims));
        }
    }

    Ok(None)
}

pub fn after_mount<M: 'static>(
    url: &Url,
    orders: &mut impl Orders<M>,
    to_msg: impl FnOnce(Msg) -> M + Clone + 'static,
) {
    match get_claims(url) {
        Ok(claims) => {
            orders.perform_cmd(async move { to_msg(Msg::LoggedIn(claims)) });
        }
        Err(e) => error!(e),
    };
}
