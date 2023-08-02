use async_recursion::async_recursion;
use clap::Parser;
use reqwest::{redirect::Policy, Client};
use std::time::Duration;
use url::Url;

#[derive(Parser)]
struct Args {
    source_url: Url,
}

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_REDIRECTS: i32 = 10;

#[async_recursion]
async fn unshorten(url: &str, tries: i32) -> Result<String, String> {
    let client = Client::builder()
        .user_agent(
            format!(
                "unshorten/{} (+https://github.com/aelew/unshorten)",
                PKG_VERSION
            )
            .as_str(),
        )
        .redirect(Policy::none())
        .build()
        .unwrap();

    let response = client.head(url).send().await.unwrap();
    let status = response.status();

    let space_count: usize = (2 + (tries * 2)).try_into().unwrap();
    let spaces = " ".repeat(space_count);

    if status.is_success() {
        println!("{}↳ Destination URL: {}", spaces, response.url());
        return Ok(url.to_string());
    }

    if !status.is_redirection() {
        return Err(format!("Failed to reach server: {status}"));
    }

    let next_url = response
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();

    if tries > MAX_REDIRECTS {
        return Err(format!("Too many redirects: {status}"));
    }

    println!(
        "{}↳ Redirecting to {} ({})",
        spaces,
        next_url,
        status.as_str()
    );

    let next_tries = tries + 1;
    unshorten(next_url, next_tries).await
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let source = args.source_url.as_str();

    println!("Source URL: {}", source);

    // Unshorten the URL using urlexpand if it's supported
    if urlexpand::is_shortened(source) {
        match urlexpand::unshorten(source, Some(Duration::from_secs(20))).await {
            Ok(destination) => {
                if source == destination {
                    println!("  ✘ The URL you provided is not shortened!");
                } else {
                    println!("  ↳ Destination URL: {}", destination);
                }
                return;
            }
            Err(e) => println!("{}", e),
        }
    }

    // Use custom implementation
    match unshorten(source, 0).await {
        Ok(destination) => {
            if source == destination {
                println!("  ✘ The URL you provided is not shortened!");
            }
        }
        Err(e) => println!("  ✘ {}", e),
    }
}
