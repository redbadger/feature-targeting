use anyhow::Result;

fn main() -> Result<()> {
    let app = todomvc_api_sqlx::create_app()?;
    smol::run(app.listen("0.0.0.0:3030"))?;
    Ok(())
}
