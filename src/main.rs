mod api_scraper;
mod http_verb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    api_scraper::do_stuff().await?;

    Ok(())
}
