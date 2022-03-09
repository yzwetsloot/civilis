use civilis::{Args, SIGTERM};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();

        println!("\nreceived Ctrl + C signal, shutting down...\n");

        unsafe {
            SIGTERM = true;
        }
    });

    let args = Args::new();
    civilis::run(args).await?;

    Ok(())
}
