#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .compact()
        .init();

    let config = ytmusic_service::config::ServiceConfig::from_env()?;
    ytmusic_service::run(config).await?;
    Ok(())
}
