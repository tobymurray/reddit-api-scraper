use reqwest;
use scraper::{Html, Selector};
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

const API_SECTION_CONTAINER_SELECTOR_STRING: &str = "div.toc > ul > li > ul > li";
const API_SECTION_API_SELECTOR_STRING: &str = concat!("div.toc > ul > li > ul > li", " > ul > li > a");

#[derive(Debug)]
enum HttpVerb {
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  TRACE,
  OPTIONS,
  CONNECT,
  PATCH,
}

impl HttpVerb {
  fn from(string: &str) -> HttpVerb {
    match string {
      "GET" => HttpVerb::GET,
      "HEAD" => HttpVerb::HEAD,
      "POST" => HttpVerb::POST,
      "PUT" => HttpVerb::PUT,
      "DELETE" => HttpVerb::DELETE,
      "TRACE" => HttpVerb::TRACE,
      "OPTIONS" => HttpVerb::OPTIONS,
      "CONNECT" => HttpVerb::CONNECT,
      "PATCH" => HttpVerb::PATCH,
      _ => panic!("No match for {}", string),
    }
  }
}

impl fmt::Display for HttpVerb {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

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
      let api = api_section.text().collect::<Vec<_>>().concat();

      let api_section_element = api_section.value();

      // E.g. #GET_wiki_{page}
      let href_to_api = api_section_element.attr("href").unwrap();
      let http_verb = strip_leading_character(word_before_underscore(href_to_api), '#');
      let http_verb = HttpVerb::from(http_verb);

      println!("    {:>3}: {} - {:#?}", j, http_verb, api);
      match http_verb {
        HttpVerb::GET => {
          write_api(http_verb, &api, &file)?;
        }
        _ => {
          println!("        Support for {} not yet implemented", http_verb);
        }
      }
    }
  }

  Ok(())
}

async fn create_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/".to_string() + filename + ".rs");
  let path = Path::new(path);
  let file = fs::File::create(path)?;
  Ok(file)
}

fn strip_leading_and_trailing_slashes(api: &str) -> &str {
  let api_without_leading_slash = strip_leading_character(api, '/');

  let last_character = api_without_leading_slash.chars().rev().next().unwrap_or_default();

  let api_without_leading_or_trailing_slash = match last_character {
    '/' => &api_without_leading_slash[..api_without_leading_slash.len() - 1],
    _ => &api_without_leading_slash,
  };

  api_without_leading_or_trailing_slash
}

fn strip_leading_character(string: &str, character: char) -> &str {
  let first_character = string.chars().next().unwrap();

  return if first_character == character {
    &string[1..]
  } else {
    string
  };
}

fn write_api(http_verb: HttpVerb, api: &str, mut file: &fs::File) -> Result<(), Box<dyn std::error::Error>> {
  let api_method_name = str::replace(strip_leading_and_trailing_slashes(api), "/", "_");
  file.write_all(("// API is: '".to_string() + api + "'\n").as_bytes())?;

  file.write_all(b"pub async fn ")?;
  file.write_all(("execute_".to_string() + &http_verb.to_string().to_lowercase() + "_").as_bytes())?;
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b"(\n")?;

  file.write_all(b"  client: &reqwest::Client,\n")?;
  file.write_all(b"  refresh_token: String,\n")?;
  file.write_all(b") -> std::result::Result<reqwest::Response, reqwest::Error> {\n")?;

  file.write_all(b"  client\n")?;
  file.write_all(("    .get(\"https://oauth.reddit.com".to_string() + api + "\")\n").as_bytes())?;
  file.write_all(b"    .bearer_auth(&refresh_token)\n")?;
  file.write_all(b"    .send()\n")?;
  file.write_all(b"    .await\n")?;

  file.write_all(b"}\n")?;
  file.write_all(b"\n")?;

  Ok(())
}

fn word_before_underscore(s: &str) -> &str {
  let bytes = s.as_bytes();

  for (i, &item) in bytes.iter().enumerate() {
    if item == b'_' {
      return &s[0..i];
    }
  }

  &s[..]
}
