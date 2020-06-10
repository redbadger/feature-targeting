use anyhow::Result;
use std::sync::RwLock;
use tide::{Redirect, Server};

mod graphql;

pub fn create_app() -> Result<Server<graphql::State>> {
    let mut app = Server::with_state(graphql::State {
        users: RwLock::new(Vec::new()),
    });
    app.at("/").get(Redirect::permanent("/graphiql"));
    app.at("/graphql").post(graphql::handle_graphql);
    app.at("/graphiql").get(graphql::handle_graphiql);
    Ok(app)
}
