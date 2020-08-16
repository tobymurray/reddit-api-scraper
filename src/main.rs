mod api_scraper;
mod http_verb;
mod openapi_models;
mod openapi_writer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // api_scraper::scrape().await?;

    openapi_writer::write();
    Ok(())
}
