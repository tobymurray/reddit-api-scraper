use std::collections::HashMap;

pub struct TemplateUri {
  pub template: String,
  pub parameters: HashMap<String, String>,
  pub request_fields: HashMap<String, String>,
}
