use crate::http_verb::HttpVerb;
use reqwest;
use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};
use std::fs;
use std::io::prelude::*;
use std::path::Path;

const API_SECTION_CONTAINER_SELECTOR_STRING: &str = "div.toc > ul > li > ul > li";
const API_SECTION_API_SELECTOR_STRING: &str = concat!("div.toc > ul > li > ul > li", " > ul > li > a");

pub async fn scrape() -> Result<(), Box<dyn std::error::Error>> {
  fs::create_dir_all("target/output/execution")?;
  fs::create_dir_all("target/output/wrapper")?;

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
    let execution_file = create_execution_file(&filename).await?;
    let wrapper_file = create_wrapper_file(&filename).await?;

    let api_section_api_selector = Selector::parse(API_SECTION_API_SELECTOR_STRING).unwrap();

    for (j, api_section) in element.select(&api_section_api_selector).enumerate() {
      let api = api_section.text().collect::<Vec<_>>().concat();

      let api_section_element = api_section.value();

      // E.g. #GET_wiki_{page}
      let href_to_api = api_section_element.attr("href").unwrap();

      let escaped_href_to_api = escape_special_characters(href_to_api);

      // println!("API is: {} and Escaped href: {}", api, escaped_href_to_api);
      let api_details_selector = Selector::parse(&escaped_href_to_api).unwrap();
      let api_details_selected = document.select(&api_details_selector);

      let uris_as_strings = get_uri_from_api_details(api_details_selected);

      let http_verb = strip_leading_character(word_before_underscore(href_to_api), '#');
      let http_verb = HttpVerb::from(http_verb);

      println!("{:>6}: {:7} - {:35} {:?}", j, http_verb, api, &uris_as_strings);

      if uris_as_strings.len() == 0 {
        println!(
          "      {} has no implementation in details, assuming it's covered elsewhere",
          api
        );
      }

      match http_verb {
        HttpVerb::GET => {
          write_api(&http_verb, &api, &execution_file)?;
          write_wrapper(&http_verb, &api, &api_section_header, &wrapper_file)?;
        }
        _ => {
          println!("        Support for {} not yet implemented", http_verb);
        }
      }
    }
  }

  Ok(())
}

async fn create_execution_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/execution/".to_string() + filename + ".rs");
  let path = Path::new(path);
  let file = fs::File::create(path)?;
  Ok(file)
}

async fn create_wrapper_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/wrapper/".to_string() + filename + ".rs");
  let path = Path::new(path);
  let file = fs::File::create(path)?;
  Ok(file)
}

fn get_uri_from_api_details(api_details: scraper::html::Select) -> Vec<String> {
  let uri_variants_selector = Selector::parse(".uri-variants li").unwrap();

  for api_detail in api_details {
    let uri_variants = api_detail.select(&uri_variants_selector);
    let mut num_variants = 0;

    for variant in uri_variants {
      num_variants = num_variants + 1;
      println!("      Variant: {:?}", variant.value());
    }

    if num_variants > 0 {
      println!(
        "      There {} {} URI {}, which {} supported yet",
        if num_variants == 1 { "is" } else { "are" },
        num_variants,
        if num_variants == 1 { "variant" } else { "variants" },
        if num_variants == 1 { "isn't" } else { "aren't" },
      );
      continue;
    } else {
      return match get_api_from_api_details(api_detail) {
        Some(api) => vec![api],
        None => Vec::new(),
      };
    }
  }
  return Vec::new();
}

fn get_api_from_api_details(api_detail: ElementRef) -> Option<String> {
  let h3_selector = Selector::parse("h3").unwrap();
  let h3_selection = api_detail.select(&h3_selector);

  let mut uri_as_string = String::new();
  for h3 in h3_selection {
    let mut uri_parts: Vec<String> = Vec::new();
    for child in h3.children() {
      match (*child.value()).as_element() {
        Some(element) => {
          if element.name() == "span" || element.name() == "a" {
            continue;
          }

          uri_parts.push(ElementRef::wrap(child).unwrap().inner_html());
        }
        _ => {
          uri_parts.push((*child.value()).as_text().unwrap().text.to_string());
        }
      }
    }
    for uri_part in uri_parts {
      uri_as_string.push_str(&uri_part);
    }
  }

  if uri_as_string.len() == 0 {
    None
  } else {
    Some(uri_as_string)
  }
}

/*
 * Trim prefixing or suffixing slashes ('/') 
 */
fn strip_leading_and_trailing_slashes(api: &str) -> &str {
  let api_without_leading_slash = strip_leading_character(api, '/');

  let last_character = api_without_leading_slash.chars().rev().next().unwrap_or_default();

  let api_without_leading_or_trailing_slash = match last_character {
    '/' => &api_without_leading_slash[..api_without_leading_slash.len() - 1],
    _ => &api_without_leading_slash,
  };

  api_without_leading_or_trailing_slash
}

/*
 * Remove the first character if it matches the provided character
 */
fn strip_leading_character(string: &str, character: char) -> &str {
  let first_character = string.chars().next().unwrap();

  return if first_character == character {
    &string[1..]
  } else {
    string
  };
}

/*
 * Some of the characters in the HTML element IDs are special characters in CSS selectors. Escape those special
 * characters so that the CSS selector will actually work instead of blowing up.
 */
fn escape_special_characters(string: &str) -> String {
  string.replace("{", "\\{").replace("}", "\\}").replace(":", "\\:")
}

fn write_api(http_verb: &HttpVerb, api: &str, mut file: &fs::File) -> Result<(), Box<dyn std::error::Error>> {
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

fn write_wrapper(
  http_verb: &HttpVerb,
  api: &str,
  api_section: &str,
  mut file: &fs::File,
) -> Result<(), Box<dyn std::error::Error>> {
  let api_method_name = str::replace(strip_leading_and_trailing_slashes(api), "/", "_");
  file.write_all(("// API is: '".to_string() + api + "'\n").as_bytes())?;

  file.write_all(b"pub async fn ")?;
  file.write_all(("wrapper_".to_string() + &http_verb.to_string().to_lowercase() + "_").as_bytes())?;
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b"(\n")?;

  file.write_all(b"  client: &reqwest::Client,\n")?;
  file.write_all(b"  client_configuration: &models::ClientConfiguration,\n")?;
  file.write_all(b"  refresh_token: &mut String,\n")?;
  file.write_all(b") -> Result<serde_json::Value, reqwest::Error> {\n")?;

  file.write_all(b"  utils::execute_with_refresh(\n")?;
  file.write_all(b"    &client,\n")?;
  file.write_all(b"    client_configuration,\n")?;
  file.write_all(b"    refresh_token,\n")?;
  file.write_all(("    ".to_string() + api_section + "::").as_bytes())?;
  file.write_all(("execute_".to_string() + &http_verb.to_string().to_lowercase() + "_").as_bytes())?;
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b",\n")?;
  file.write_all(b"  )\n")?;
  file.write_all(b"  .await\n")?;

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
