use tonic::transport::Server;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = server::Service::default();

    Server::builder()
        .add_service(server::HandleFeatureTargetingServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
