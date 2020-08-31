mod api_scraper;
mod generator;
mod http_verb;
mod template_uri;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = reqwest::get("https://www.reddit.com/dev/api").await?.text().await?;

    api_scraper::scrape(&html).await?;

    Ok(())
}
