use std::fmt;

#[derive(Debug)]
pub enum HttpVerb {
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
  pub fn from(string: &str) -> HttpVerb {
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
