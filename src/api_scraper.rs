use reqwest;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::prelude::*;

const API_SECTION_CONTAINER_SELECTOR_STRING: &str = "div.toc > ul > li > ul > li";
const API_SECTION_API_SELECTOR_STRING: &str =
  concat!("div.toc > ul > li > ul > li", " > ul > li > a");

pub async fn do_stuff() -> Result<(), Box<dyn std::error::Error>> {
  let resp = reqwest::get("https://www.reddit.com/dev/api")
    .await?
    .text()
    .await?;

  let document = Html::parse_document(&resp);

  let div_sidebar_selector = Selector::parse("div.content div.sidebar").unwrap();
  let div_sidebar = document.select(&div_sidebar_selector).next().unwrap();

  let api_section_container_selector =
    Selector::parse(API_SECTION_CONTAINER_SELECTOR_STRING).unwrap();
  let api_section_container = div_sidebar.select(&api_section_container_selector);

  println!(
    "Number of elements found: {}",
    api_section_container.clone().count()
  );

  let api_section_header_selector = Selector::parse("a").unwrap();

  for (i, element) in api_section_container.enumerate() {
    let api_section_header = element
      .select(&api_section_header_selector)
      .next()
      .unwrap()
      .text()
      .collect::<Vec<_>>()[0];
    if i > 1 {
      continue;
    }
    println!("Section {}: {}", i, api_section_header);
    create_file(api_section_header).await?;

    // println!("Going to use selector: {}", &(API_SECTION_CONTAINER_SELECTOR_STRING.to_string() + "> [href$='#section_" + api_section_header + "']"));

    let api_section_api_selector = Selector::parse(API_SECTION_API_SELECTOR_STRING).unwrap();

    for (j, api_section) in element.select(&api_section_api_selector).enumerate() {
      // if j == 0 {
      let api = api_section.value();
      println!("    {}: {:#?}", j, api);
      // }
    }
  }

  Ok(())
}

async fn create_file(filename: &str) -> std::io::Result<()> {
  let mut file = File::create(filename)?;
  file.write_all(b"Hello, world!")?;
  Ok(())
}
