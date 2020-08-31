use crate::generator;
use crate::http_verb::HttpVerb;
use crate::template_uri;

use regex::Regex;
use scraper::element_ref::ElementRef;
use scraper::{Html, Selector};
use std::collections::HashMap;

const API_SECTION_CONTAINER_SELECTOR_STRING: &str = "div.toc > ul > li > ul > li";
const API_SECTION_API_SELECTOR_STRING: &str = concat!("div.toc > ul > li > ul > li", " > ul > li > a");

pub async fn scrape(html: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    generator::generate().await?;
    let execution_file = generator::create_execution_file(&filename).await?;
    let wrapper_file = generator::create_wrapper_file(&filename).await?;
    let request_model_file = generator::create_request_model_file(&filename).await?;

    let api_section_api_selector = Selector::parse(API_SECTION_API_SELECTOR_STRING).unwrap();

    for (_, api_section) in element.select(&api_section_api_selector).enumerate() {
      let api_section_element = api_section.value();

      // E.g. #GET_wiki_{page}
      let href_to_api = api_section_element.attr("href").unwrap();

      let escaped_href_to_api = escape_special_characters(href_to_api);

      // println!("API is: {} and Escaped href: {}", api, escaped_href_to_api);
      let api_details_selector = Selector::parse(&escaped_href_to_api).unwrap();
      let api_details_selected = document.select(&api_details_selector);

      let http_verb = word_before_underscore(href_to_api).trim_start_matches('#');
      let http_verb = HttpVerb::from(http_verb);

      let uris_as_strings = get_uri_from_api_details(api_details_selected, &http_verb);

      for uri in uris_as_strings {
        match http_verb {
          HttpVerb::GET => {
            generator::write_api(&http_verb, &uri, &execution_file)?;
            generator::write_wrapper(&http_verb, &uri, &api_section_header, &wrapper_file)?;
          }
          HttpVerb::POST => {
            generator::write_api(&http_verb, &uri, &execution_file)?;
            generator::write_wrapper(&http_verb, &uri, &filename, &wrapper_file)?;
            generator::write_request_model_file(&uri, &request_model_file)?;
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

fn get_uri_from_api_details(
  api_details: scraper::html::Select,
  http_verb: &HttpVerb,
) -> Vec<template_uri::TemplateUri> {
  let uri_variants_selector = Selector::parse(".uri-variants li").unwrap();

  let api_detail = api_details.enumerate().next();

  if api_detail == None {
    return Vec::new();
  }

  // There should be only one, assume it's that way for now
  let api_detail = api_detail.unwrap().1;

  let uri_variants = api_detail.select(&uri_variants_selector);
  let mut num_variants = 0;

  let request_fields = match http_verb {
    HttpVerb::POST => get_request_body_from_api_details(api_detail),
    _ => HashMap::new(),
  };

  let mut variants: Vec<template_uri::TemplateUri> = Vec::new();
  for variant in uri_variants {
    num_variants = num_variants + 1;
    variants.extend(uri_prototype_into_concrete(
      &collect_children_as_string(variant)
        .unwrap()
        .trim_start_matches("→")
        .trim()
        .to_string(),
      request_fields.clone(),
    ));
  }

  if num_variants > 0 {
    return variants;
  }

  return match get_api_from_api_details(api_detail) {
    Some(api) => uri_prototype_into_concrete(&api, request_fields.clone()),
    None => Vec::new(),
  };
}

fn get_request_body_from_api_details(api_detail: ElementRef) -> HashMap<String, String> {
  let parameter_row_selector = Selector::parse("table.parameters > tbody > tr").unwrap();
  let parameter_description_selector = Selector::parse("td > p").unwrap();
  let parameter_name_selector = Selector::parse("th").unwrap();
  let parameter_row_selection = api_detail.select(&parameter_row_selector);

  let mut request_fields = HashMap::new();
  for selection in parameter_row_selection {
    let parameter_name = selection.select(&parameter_name_selector).next().unwrap();
    let parameter_description = selection.select(&parameter_description_selector).next();
    request_fields.insert(
      parameter_name.inner_html(),
      match parameter_description {
        Some(description) => description.inner_html(),
        None => "".to_string(),
      },
    );
  }

  return request_fields;
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
fn uri_prototype_into_concrete(
  prototype: &str,
  request_fields: HashMap<String, String>,
) -> Vec<template_uri::TemplateUri> {
  let uri_variant_section = Regex::new(r"\[(.*)\]").unwrap();
  let uri_parameter = Regex::new(r"(\{\{(\w+)\}\})").unwrap();

  if uri_variant_section.is_match(prototype) {
    let uri_without_section = uri_variant_section.replace_all(prototype, "").to_string();

    let mut uri_without_section_parameters = HashMap::new();
    for parameter_match in uri_parameter.captures_iter(&uri_without_section) {
      uri_without_section_parameters.insert(parameter_match[1].to_string(), parameter_match[2].to_string());
    }

    // Assume there's a single section for now ([/r/subreddit])
    let uri_without_section = template_uri::TemplateUri {
      template: uri_without_section,
      parameters: uri_without_section_parameters,
      request_fields: request_fields.clone(),
    };

    let uri_with_section = uri_variant_section.replace_all(prototype, "$1").to_string();
    let mut uri_with_section_parameters = HashMap::new();
    for parameter_match in uri_parameter.captures_iter(&uri_with_section) {
      uri_with_section_parameters.insert(parameter_match[1].to_string(), parameter_match[2].to_string());
    }

    let uri_with_section = template_uri::TemplateUri {
      template: uri_with_section,
      parameters: uri_with_section_parameters,
      request_fields: request_fields.clone(),
    };

    vec![uri_without_section, uri_with_section]
  } else {
    let mut parameters = HashMap::new();
    for parameter_match in uri_parameter.captures_iter(prototype) {
      parameters.insert(parameter_match[1].to_string(), parameter_match[2].to_string());
    }

    vec![template_uri::TemplateUri {
      template: uri_variant_section.replace_all(prototype, "$1").to_string(),
      parameters: parameters,
      request_fields: request_fields.clone(),
    }]
  }
}
