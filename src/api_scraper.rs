use reqwest;
use scraper::{Html, Selector};
use std::fs;
use std::io::prelude::*;
use std::path::Path;

const API_SECTION_CONTAINER_SELECTOR_STRING: &str = "div.toc > ul > li > ul > li";
const API_SECTION_API_SELECTOR_STRING: &str = concat!("div.toc > ul > li > ul > li", " > ul > li > a");

pub async fn do_stuff() -> Result<(), Box<dyn std::error::Error>> {
  fs::create_dir_all("target/output")?;

  let resp = reqwest::get("https://www.reddit.com/dev/api").await?.text().await?;

  let document = Html::parse_document(&resp);

  let div_sidebar_selector = Selector::parse("div.content div.sidebar").unwrap();
  let div_sidebar = document.select(&div_sidebar_selector).next().unwrap();

  let api_section_container_selector = Selector::parse(API_SECTION_CONTAINER_SELECTOR_STRING).unwrap();
  let api_section_container = div_sidebar.select(&api_section_container_selector);

  println!("Number of elements found: {}", api_section_container.clone().count());

  let api_section_header_selector = Selector::parse("a").unwrap();

  for (i, element) in api_section_container.enumerate() {
    let api_section_header = element
      .select(&api_section_header_selector)
      .next()
      .unwrap()
      .text()
      .collect::<Vec<_>>()[0];

    println!("Section {}: {}", i, api_section_header);

    let filename = str::replace(api_section_header, "&", "and");
    let filename = str::replace(&filename, " ", "_");
    let file = create_file(&filename).await?;

    let api_section_api_selector = Selector::parse(API_SECTION_API_SELECTOR_STRING).unwrap();

    for (j, api_section) in element.select(&api_section_api_selector).enumerate() {
      let api = api_section.text().collect::<Vec<_>>()[0];
      println!("    {}: {:#?}", j, api);
      write_api(api, &file)?;
    }
  }

  Ok(())
}

async fn create_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/".to_string() + filename + ".rs");
  let path = Path::new(path);
  println!("    Creating in path {}", path.display());
  let file = fs::File::create(path)?;
  Ok(file)
}

fn strip_leading_and_trailing_slashes(api: &str) -> &str {
  let api_without_leading_slash = match api.chars().next().unwrap() {
    '/' => &api[1..],
    _ => &api,
  };

  let last_character = api_without_leading_slash.chars().rev().next().unwrap_or_default();

  let api_without_leading_or_trailing_slash = match last_character {
    '/' => &api_without_leading_slash[..api_without_leading_slash.len() - 1],
    _ => &api_without_leading_slash,
  };

  api_without_leading_or_trailing_slash
}

fn write_api(api: &str, mut file: &fs::File) -> Result<(), Box<dyn std::error::Error>> {
  file.write_all(b"// API is: '")?;
  file.write_all(api.as_bytes())?;
  file.write_all(b"'\n")?;

  let api_without_leading_or_trailing_slash = strip_leading_and_trailing_slashes(api);

  file.write_all(b"pub fn ")?;

  let api_method_name = str::replace(api_without_leading_or_trailing_slash, "/", "_");
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b"() {\n")?;
  file.write_all(b"  println!(\"")?;
  file.write_all(api.as_bytes())?;
  file.write_all(b"\");\n")?;
  file.write_all(b"}\n")?;

  file.write_all(b"\n")?;

  Ok(())
}
