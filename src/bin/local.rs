use reqwest::Client;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let mut handles = Vec::new();

    let client = Client::new();

    let start = Instant::now();

    for _ in 1..1000 {
        let client = client.clone();

        let handle = tokio::spawn(async move { get(&client).await });
        handles.push(handle);
    }

    for task in handles {
        task.await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("took {:?}", elapsed);
}

async fn get(client: &reqwest::Client) {
    match client.get("http://localhost:8080/").send().await {
        Ok(res) => {
            println!("{}", res.status());
            let _ = res.text().await;
        }
        Err(error) => eprintln!("{}", error),
    };
}
