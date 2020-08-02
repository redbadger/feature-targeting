use anyhow::Result;
use auth::Claims;
use seed::{prelude::*, *};

mod auth;

pub enum Msg {
    Login,
    Logout,
    LoggedIn(Option<Claims>),
}

pub struct Model {
    pub base_url: Url,
    pub jwt: Option<String>,
    pub user: Option<String>,
}

impl Model {
    pub fn new(base_url: Url, jwt: Option<String>, user: Option<String>) -> Self {
        Self {
            base_url,
            jwt,
            user,
        }
    }
}

pub fn update(msg: Msg, model: &mut Model) {
    use Msg::*;
    match msg {
        Login => 
        LoggedIn(Some(claims)) => model.user = Some(claims.name),
        LoggedIn(None) => model.user = None,
    }
}

pub fn view<M: 'static>(
    user: &Option<String>,
    url: &Url,
    _to_msg: impl FnOnce(Msg) -> M + Clone + 'static,
) -> Node<M> {
    nav![
        C!["auth-text"],
        if let Some(user) = user {
            nodes![
                span![format!("{} ", user)],
                a![
                    C!["auth-link"],
                    attrs! {
                        At::Href => format!("{}logout", url.to_base_url())
                    },
                    "logout"
                ]
            ]
        } else {
            nodes![
                span!["Please "],
                a![
                    C!["auth-link"],
                    attrs! {
                        At::Href => format!("{}login", url.to_base_url())
                    },
                    "login"
                ],
                span![" to modify todos"]
            ]
        },
    ]
}

pub fn get_claims() -> Result<Option<Claims>> {
    if let Some(response) = super::cookies::get_cookie("token") {
        let claims = auth::decode_jwt(&response)?;
        return Ok(Some(claims));
    }

    Ok(None)
}

pub fn after_mount<M: 'static>(
    orders: &mut impl Orders<M>,
    to_msg: impl FnOnce(Msg) -> M + Clone + 'static,
) {
    match get_claims() {
        Ok(claims) => {
            orders.perform_cmd(async move { to_msg(Msg::LoggedIn(claims)) });
        }
        Err(e) => error!(e),
    };
}
