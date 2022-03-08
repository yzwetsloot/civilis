use civilis::Args;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::new();
    civilis::run(args).await?;

    Ok(())
}
