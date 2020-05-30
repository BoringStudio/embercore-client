use embercore_client_lib::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new("configs/client.local.yml")?;

    embercore_client_lib::run(config).await?;

    Ok(())
}
