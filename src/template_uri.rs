use std::collections::HashMap;
use std::fmt;

pub struct TemplateUri {
  pub template: String,
  pub parameters: HashMap<String, String>,
  pub request_fields: HashMap<String, String>,  
}

impl fmt::Display for TemplateUri {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}, {:?}", self.template, self.parameters)
  }
}
