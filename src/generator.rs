use crate::http_verb::HttpVerb;
use crate::template_uri;

use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::str;

pub fn write_get_api(api: &template_uri::TemplateUri, mut file: &fs::File) -> Result<(), Box<dyn std::error::Error>> {
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

  let mut handlebars = Handlebars::new();
  handlebars.set_strict_mode(true);

  let mut parameters: HashMap<String, String> = HashMap::new();
  parameters.insert("api_path".to_string(), api.template.clone());
  parameters.insert("api_name".to_string(), api_method_name);

  if !api.parameters.is_empty() {
    parameters.insert("parameters".to_string(), "true".to_string());
  }

  let bytes = include_bytes!("handlebars/http_get.handlebars");
  let handlebars_template = str::from_utf8(bytes).unwrap();
  let handlebars_template = handlebars.render_template(handlebars_template, &parameters).unwrap();

  file.write_all(handlebars_template.as_bytes()).unwrap();

  Ok(())
}

pub fn write_post_api(
  http_verb: &HttpVerb,
  api: &template_uri::TemplateUri,
  mut file: &fs::File,
) -> Result<(), Box<dyn std::error::Error>> {
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

  if api.request_fields.is_empty() {
    file.write_all(b"  _request_fields: &HashMap<String, String>,\n")?;
  } else {
    file.write_all(b"  request_fields: &HashMap<String, String>,\n")?;
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
        + "(&(\"https://oauth.reddit.com\".to_string() + &handlebars.render_template(\""
        + &api.template
        + "\", &parameters).unwrap()))\n")
        .as_bytes(),
    )?;
  }
  match http_verb {
    HttpVerb::POST => {
      if !api.request_fields.is_empty() {
        file.write_all(b"    .json(&request_fields)\n")?;
      }
    }
    HttpVerb::GET => {
      // file.write_all("  utils::execute_get_api("")
    }
    _ => {}
  }

  // utils::execute_get_api("/api/v1/me", client, refresh_token).await
  file.write_all(b"    .bearer_auth(&refresh_token)\n")?;
  file.write_all(b"    .send()\n")?;
  file.write_all(b"    .await\n")?;

  file.write_all(b"}\n")?;
  file.write_all(b"\n")?;

  Ok(())
}

pub fn write_wrapper(
  http_verb: &HttpVerb,
  api: &template_uri::TemplateUri,
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

  let structure_name = &api
    .template
    .trim_start_matches('/')
    .trim_end_matches('/')
    .replace("{", "")
    .replace("}", "")
    .replace("/", "_")
    .to_ascii_uppercase();

  file.write_all(("// API is: '".to_string() + &api.template + "'\n").as_bytes())?;

  file.write_all(b"pub async fn ")?;
  file.write_all(("wrapper_".to_string() + &http_verb.to_string().to_lowercase() + "_").as_bytes())?;
  file.write_all(api_method_name.as_bytes())?;
  file.write_all(b"(\n")?;

  file.write_all(b"  client: &reqwest::Client,\n")?;
  file.write_all(b"  client_configuration: &client::ClientConfiguration,\n")?;
  file.write_all(b"  refresh_token: &mut String,\n")?;
  if !api.parameters.is_empty() {
    if api.template == "/hot" {
      println!("Parameters are: {:?}", api.parameters);
    }
    file.write_all(b"  parameters: &HashMap<String, String>,\n")?;
  }

  if !api.request_fields.is_empty() {
    file.write_all(("  request_fields: ".to_string() + structure_name + ",\n").as_bytes())?
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
  file.write_all(b"    &HashMap::new(),\n")?;
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

pub fn write_request_model_file(
  api: &template_uri::TemplateUri,
  mut file: &fs::File,
) -> Result<(), Box<dyn std::error::Error>> {
  let structure_name = &api
    .template
    .trim_start_matches('/')
    .trim_end_matches('/')
    .replace("{", "")
    .replace("}", "")
    .replace("/", "_")
    .to_ascii_uppercase();

  file.write_all(("// API is: '".to_string() + &api.template + "'\n").as_bytes())?;
  file.write_all(("pub struct ".to_string() + structure_name + " {\n").as_bytes())?;
  for field in api.request_fields.clone() {
    if !field.1.is_empty() {
      file.write_all(("  // ".to_string() + &field.1 + "\n").as_bytes())?;
    }

    file.write_all(("  ".to_string() + &field.0 + ": String,\n\n").as_bytes())?;
  }
  file.write_all(b"}")?;

  Ok(())
}

pub async fn generate() -> Result<(), Box<dyn std::error::Error>> {
  fs::create_dir_all("target/output/execution")?;
  fs::create_dir_all("target/output/wrapper")?;
  fs::create_dir_all("target/output/request_models")?;

  Ok(())
}

pub async fn create_execution_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/execution/".to_string() + filename + ".rs");
  let path = Path::new(path);
  let file = fs::File::create(path)?;
  Ok(file)
}

pub async fn create_wrapper_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/wrapper/".to_string() + filename + ".rs");
  let path = Path::new(path);
  let file = fs::File::create(path)?;
  Ok(file)
}

pub async fn create_request_model_file(filename: &str) -> std::io::Result<fs::File> {
  let path = &("./target/output/request_models/".to_string() + filename + ".rs");
  let path = Path::new(path);
  let file = fs::File::create(path)?;
  Ok(file)
}
