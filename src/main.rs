mod api_scraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    api_scraper::do_stuff().await?;

    Ok(())
}
