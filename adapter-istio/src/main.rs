use tonic::transport::Server;
mod server;

const PORT: u16 = 50051;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!(
        "0.0.0.0:{}",
        std::env::var("PORT").unwrap_or_else(|e| {
            eprintln!("defaulting PORT to {} ({})", PORT, e);
            PORT.to_string()
        })
    )
    .parse()?;
    let svc = server::Service::default();

    Server::builder()
        .add_service(server::HandleFeatureTargetingServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
