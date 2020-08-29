use crate::http_verb::HttpVerb;
use regex::Regex;
use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

const API_SECTION_CONTAINER_SELECTOR_STRING: &str = "div.toc > ul > li > ul > li";
const API_SECTION_API_SELECTOR_STRING: &str = concat!("div.toc > ul > li > ul > li", " > ul > li > a");

struct TemplateUri {
  template: String,
  parameters: HashMap<String, String>,
}

pub async fn scrape(html: &str) -> Result<(), Box<dyn std::error::Error>> {
  fs::create_dir_all("target/output/execution")?;
  fs::create_dir_all("target/output/wrapper")?;

  let document = Html::parse_document(html);

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

      let http_verb = word_before_underscore(href_to_api).trim_start_matches('#');
      let http_verb = HttpVerb::from(http_verb);

      // println!("{:>6}: {:7} - {:35} {:?}", j, http_verb, api, &uris_as_strings);

      if uris_as_strings.len() == 0 {
        println!(
          "      {} has no implementation in details, assuming it's covered elsewhere",
          api
        );
      }

      for uri in uris_as_strings {
        match http_verb {
          HttpVerb::GET => {
            write_api(&http_verb, &uri, &execution_file)?;
            write_wrapper(&http_verb, &uri, &api_section_header, &wrapper_file)?;
          }
          HttpVerb::POST => {
            write_api(&http_verb, &uri, &execution_file)?;
            write_wrapper(&http_verb, &uri, &filename, &wrapper_file)?;
          }
          _ => {
            println!("        Support for {} not yet implemented", http_verb);
          }
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

fn get_uri_from_api_details(api_details: scraper::html::Select) -> Vec<TemplateUri> {
  let uri_variants_selector = Selector::parse(".uri-variants li").unwrap();

  // There should be only one, assume it's that way for now
  let api_detail = api_details.enumerate().next().unwrap().1;

  let uri_variants = api_detail.select(&uri_variants_selector);
  let mut num_variants = 0;

  let mut variants: Vec<TemplateUri> = Vec::new();
  for variant in uri_variants {
    num_variants = num_variants + 1;
    variants.extend(uri_prototype_into_concrete(
      &collect_children_as_string(variant)
        .unwrap()
        .trim_start_matches("→")
        .trim()
        .to_string(),
    ));
  }

  if num_variants > 0 {
    return variants;
  }

  return match get_api_from_api_details(api_detail) {
    Some(api) => uri_prototype_into_concrete(&api),
    None => Vec::new(),
  };
}

fn get_api_from_api_details(api_detail: ElementRef) -> Option<String> {
  let h3_selector = Selector::parse("h3").unwrap();
  let h3_selection = api_detail.select(&h3_selector);

  for h3 in h3_selection {
    // Assuming there's only one...
    return collect_children_as_string(h3);
  }

  None
}

fn collect_children_as_string(parent: ElementRef) -> Option<String> {
  let mut uri_parts: Vec<String> = Vec::new();
  for child in parent.children() {
    match (*child.value()).as_element() {
      Some(element) => {
        if element.name() == "span" || element.name() == "a" {
          continue;
        }

        let element_ref = ElementRef::wrap(child).unwrap();
        if element.name() == "em" {
          uri_parts.push("{{".to_string() + &element_ref.inner_html().to_string() + "}}");
        } else {
          uri_parts.push(element_ref.inner_html());
        }
      }
      _ => {
        uri_parts.push((*child.value()).as_text().unwrap().text.to_string());
      }
    }
  }

  let mut uri_as_string = String::new();
  for uri_part in uri_parts {
    uri_as_string.push_str(&uri_part);
  }

  if uri_as_string.len() == 0 {
    None
  } else {
    Some(uri_as_string)
  }
}

/*
 * Some of the characters in the HTML element IDs are special characters in CSS selectors. Escape those special
 * characters so that the CSS selector will actually work instead of blowing up.
 */
fn escape_special_characters(string: &str) -> String {
  string.replace("{", "\\{").replace("}", "\\}").replace(":", "\\:")
}

fn write_api(http_verb: &HttpVerb, api: &TemplateUri, mut file: &fs::File) -> Result<(), Box<dyn std::error::Error>> {
  file.write_all(("// API is: '".to_string() + &api.template + "'\n").as_bytes())?;

  let api_method_name = str::replace(
    &api
      .template
      .trim_start_matches('/')
      .trim_end_matches('/')
      .replace("{", "")
      .replace("}", ""),
    "/",
    "_",
  );
  file.write_all(b"pub async fn ")?;
  file.write_all(("execute_".to_string() + &http_verb.to_string().to_lowercase() + "_").as_bytes())?;
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b"(\n")?;

  file.write_all(b"  client: &reqwest::Client,\n")?;
  file.write_all(b"  refresh_token: String,\n")?;
  if api.parameters.is_empty() {
    file.write_all(b"  _parameters: &HashMap<String, String>,\n")?;
  } else {
    file.write_all(b"  parameters: &HashMap<String, String>,\n")?;
  }
  file.write_all(b") -> std::result::Result<reqwest::Response, reqwest::Error> {\n")?;

  // We'll need handlebars or templating
  if !api.parameters.is_empty() {
    file.write_all(b"  let mut handlebars = Handlebars::new();\n")?;
    file.write_all(b"  handlebars.set_strict_mode(true);\n")?;
  }

  file.write_all(b"  client\n")?;
  if api.parameters.is_empty() {
    file.write_all(
      ("    .".to_string()
        + &http_verb.to_string().to_lowercase()
        + "(\"https://oauth.reddit.com"
        + &api.template
        + "\")\n")
        .as_bytes(),
    )?;
  } else {
    file.write_all(
      ("    .".to_string()
        + &http_verb.to_string().to_lowercase()
        + "(\"https://oauth.reddit.com\".to_string() + &handlebars.render_template(\""
        + &api.template
        + "\", &parameters).unwrap()))\n")
        .as_bytes(),
    )?;
  }
  file.write_all(b"    .bearer_auth(&refresh_token)\n")?;
  file.write_all(b"    .send()\n")?;
  file.write_all(b"    .await\n")?;

  file.write_all(b"}\n")?;
  file.write_all(b"\n")?;

  Ok(())
}

fn write_wrapper(
  http_verb: &HttpVerb,
  api: &TemplateUri,
  api_section: &str,
  mut file: &fs::File,
) -> Result<(), Box<dyn std::error::Error>> {
  let api_method_name = str::replace(
    &api
      .template
      .trim_start_matches('/')
      .trim_end_matches('/')
      .replace("{", "")
      .replace("}", ""),
    "/",
    "_",
  );
  file.write_all(("// API is: '".to_string() + &api.template + "'\n").as_bytes())?;

  file.write_all(b"pub async fn ")?;
  file.write_all(("wrapper_".to_string() + &http_verb.to_string().to_lowercase() + "_").as_bytes())?;
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b"(\n")?;

  file.write_all(b"  client: &reqwest::Client,\n")?;
  file.write_all(b"  client_configuration: &models::ClientConfiguration,\n")?;
  file.write_all(b"  refresh_token: &mut String,\n")?;
  if !api.parameters.is_empty() {
    file.write_all(b"  parameters: &HashMap<String, String>,\n")?;
  }
  file.write_all(b") -> Result<serde_json::Value, reqwest::Error> {\n")?;

  file.write_all(b"  utils::execute_with_refresh(\n")?;
  file.write_all(b"    &client,\n")?;
  file.write_all(b"    client_configuration,\n")?;
  file.write_all(b"    refresh_token,\n")?;
  if api.parameters.is_empty() {
    file.write_all(b"    &HashMap::new(),\n")?;
  } else {
    file.write_all(b"    parameters,\n")?;
  }
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

/*
 * A bunch of APIs are of the form [/r/subreddit]/about/banned where the API works as written but
 * also without the /r/subreddit prefix, so there are actually two APIS ('/about/banned' and
 * '/r/subreddit/about/banned')
 */
fn uri_prototype_into_concrete(prototype: &str) -> Vec<TemplateUri> {
  let uri_variant_section = Regex::new(r"\[(.*)\]").unwrap();
  let uri_parameter = Regex::new(r"(\{\{(\w+)\}\})").unwrap();

  if uri_variant_section.is_match(prototype) {
    let uri_without_section = uri_variant_section.replace_all(prototype, "").to_string();

    let mut uri_without_section_parameters = HashMap::new();
    for parameter_match in uri_parameter.captures_iter(&uri_without_section) {
      uri_without_section_parameters.insert(parameter_match[1].to_string(), parameter_match[2].to_string());
    }
    // Assume there's a single section for now ([/r/subreddit])
    let uri_without_section = TemplateUri {
      template: uri_without_section,
      parameters: uri_without_section_parameters,
    };

    let uri_with_section = uri_variant_section.replace_all(prototype, "$1").to_string();
    let mut uri_with_section_parameters = HashMap::new();
    for parameter_match in uri_parameter.captures_iter(&uri_with_section) {
      uri_with_section_parameters.insert(parameter_match[1].to_string(), parameter_match[2].to_string());
    }

    let uri_with_section = TemplateUri {
      template: uri_with_section,
      parameters: uri_with_section_parameters,
    };

    vec![uri_without_section, uri_with_section]
  } else {
    let mut parameters = HashMap::new();
    for parameter_match in uri_parameter.captures_iter(prototype) {
      parameters.insert(parameter_match[1].to_string(), parameter_match[2].to_string());
    }

    vec![TemplateUri {
      template: uri_variant_section.replace_all(prototype, "$1").to_string(),
      parameters: parameters,
    }]
  }
}
