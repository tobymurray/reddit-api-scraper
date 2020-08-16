use crate::openapi_models::Info;
use crate::openapi_models::OpenApiDocument;
use crate::openapi_models::PathItem;
use crate::openapi_models::ApiPath;

use serde_json::Result;

pub fn write() -> Result<()> {
  let info = Info {
    title: "Mr. Splashy Pants".to_string(),
    version: "0.1.1".to_string(),
  };

  let path = ApiPath {
    path: "/this/is/a/test".to_string(),
    details: PathItem {

    },
  };

  let mut paths = serde_json::Map::new();
  paths.insert(path.path, serde_json::to_value(path.details)?);

  let openapi_document = OpenApiDocument {
    openapi: "3.0.3".to_string(),
    info: info,
    paths: serde_json::Value::Object(paths),
  };

  let string = serde_json::to_string_pretty(&openapi_document)?;
  println!("{}", string);

  Ok(())
}
