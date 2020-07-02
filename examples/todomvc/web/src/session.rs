use super::{auth, Model};
use anyhow::anyhow;
use seed::{prelude::*, *};

const STORAGE_KEY: &str = "todomvc-session";

pub enum Msg {
    Login,
    LoggedIn(Option<auth::Claims>),
    Logout,
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<super::Msg>) {
    let data = &mut model.data;
    use Msg::*;
    match msg {
        Login => {
            if let Ok(url) = auth::get_login_url() {
                window()
                    .open_with_url_and_target(url.to_string().as_str(), "_self")
                    .expect("couldn't open window");
            }
            orders.skip();
        }
        LoggedIn(Some(claims)) => {
            data.user = Some(claims.name);
            Url::new()
                .set_path(model.base_url.hash_path())
                .go_and_replace();
        }
        LoggedIn(None) => {
            data.user = None;
        }

        Logout => {
            if let Ok(()) = auth::logout() {
                LocalStorage::remove(STORAGE_KEY).expect("problem removing key from local storage");
                data.user = None;
            }
        }
    }
}

pub fn view_nav(user: &Option<String>) -> Node<super::Msg> {
    nav![
        C!["auth-text"],
        if let Some(user) = user {
            nodes![
                span![format!("{} ", user)],
                a![
                    C!["auth-link"],
                    mouse_ev(Ev::Click, |_| super::Msg::Session(Msg::Logout)),
                    "logout"
                ]
            ]
        } else {
            nodes![
                span!["Please "],
                a![
                    C!["auth-link"],
                    mouse_ev(Ev::Click, |_| super::Msg::Session(Msg::Login)),
                    "login"
                ],
                span![" to modify todos"]
            ]
        },
    ]
}

pub fn get_claims(url: &Url) -> anyhow::Result<Option<auth::Claims>> {
    if let Ok(response) = LocalStorage::get(STORAGE_KEY) {
        let claims = auth::get_claims(&response)?;
        return Ok(Some(claims));
    } else {
        let url = url.to_string();
        let url = url
            .strip_prefix("/#")
            .ok_or_else(|| anyhow!("url doesn't start with \"/#\""))?;
        if let Ok(response) = serde_urlencoded::from_str::<auth::AuthResponse>(url) {
            let claims = auth::get_claims(&response)?;
            LocalStorage::insert(STORAGE_KEY, &response)
                .map_err(|e| anyhow!("cannot insert into localstorage {:?}", e))?;
            return Ok(Some(claims));
        }
    }
    Ok(None)
}

pub fn after_mount(url: &Url, orders: &mut impl Orders<super::Msg>) {
    match get_claims(url) {
        Ok(claims) => {
            orders.perform_cmd(async move { super::Msg::Session(Msg::LoggedIn(claims)) });
        }
        Err(e) => error!(e),
    };
}
