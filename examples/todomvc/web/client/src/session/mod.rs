use anyhow::Result;
use auth::Claims;
use seed::{prelude::*, *};

mod auth;

struct_urls!();
impl<'a> Urls<'a> {
    pub fn login(self) -> Url {
        self.base_url().add_path_part("login")
    }
    pub fn logout(self) -> Url {
        self.base_url().add_path_part("logout")
    }
}

pub enum Msg {
    Login,
    Logout,
    LoggedIn(Option<Claims>),
}

pub struct Model {
    pub base_url: Url,
    login_url: Url,
    logout_url: Url,
    pub jwt: Option<String>,
    pub user: Option<String>,
}

impl Model {
    pub fn new(base_url: &Url, jwt: Option<String>, user: Option<String>) -> Self {
        Self {
            base_url: base_url.clone(),
            login_url: Urls::new(base_url).login(),
            logout_url: Urls::new(base_url).logout(),
            jwt,
            user,
        }
    }
}

pub fn update(msg: Msg, model: &mut Model) {
    use Msg::*;
    match msg {
        Login => model.login_url.go_and_load(),
        LoggedIn(Some(claims)) => model.user = Some(claims.name),
        LoggedIn(None) => model.user = None,
        Logout => model.logout_url.go_and_load(),
    }
}

pub fn view<M: 'static>(model: &Model, to_msg: impl FnOnce(Msg) -> M + Clone + 'static) -> Node<M> {
    nav![
        C!["auth-text"],
        if let Some(user) = &model.user {
            nodes![
                span![format!("{} ", user)],
                a![
                    C!["auth-link"],
                    attrs! {
                        At::Href => model.logout_url
                    },
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
                        At::Href => model.login_url
                    },
                    mouse_ev(Ev::Click, |_| to_msg(Msg::Login)),
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

pub fn init<M: 'static>(
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
