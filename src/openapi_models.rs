#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Operation {
  pub get: 
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PathItem {
  pub get: 
}

pub struct ApiPath {
  pub path: String,
  pub details: PathItem,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Info {
  pub title: String,
  pub version: String,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OpenApiDocument {
  pub openapi: String,
  pub info: Info,
  pub paths: serde_json::Value,
}
