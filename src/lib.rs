pub mod rules;
pub mod config {
  use crate::rules::Rule;
  use hyper::Uri;
  use lazy_static::lazy_static;
  use regex::Regex;
  use std::collections::HashMap;
  use std::fs;
  use std::path::Path;

  pub fn parse_rules_from<P: AsRef<Path>>(path: P) -> HashMap<String, Box<dyn Rule>> {
    lazy_static! {
      static ref RULE_RE: Regex = Regex::new(r#"^(.+)\b=\b(.+)"#).unwrap();
    }
    let data = fs::read_to_string(path).unwrap();
    let config_rules = data.trim().split("\n").map(|line| {
      let captures = RULE_RE
        .captures(line)
        .expect("Malformed config URL: expecte (kw) = (url)");
      ConfigRule::new(&captures[1], &captures[2])
    });
    let mut rules: HashMap<String, Box<dyn Rule>> = HashMap::new();
    for cfg_rule in config_rules {
      rules.insert(cfg_rule.kw().to_string(), Box::new(cfg_rule));
    }
    rules
  }

  #[derive(Default)]
  struct ConfigRule {
    kw: String,
    uri: String,
  }

  impl ConfigRule {
    pub fn new<K, U>(kw: K, uri: U) -> Self
    where
      K: Into<String>,
      U: Into<String>,
    {
      Self {
        kw: kw.into(),
        uri: uri.into(),
      }
    }

    pub fn kw(&self) -> &str {
      &self.kw
    }
  }
  // TODO: More
  impl Rule for ConfigRule {
    // Alter args such that the cmd name is the first arg
    fn produce_uri(&self, cmd: String, args: &Vec<String>) -> Result<Uri, String> {
      static ARGS_STR: &'static str = "{ARGS}";
      static ALL_STR: &'static str = "{ALL}";
      let mut uri = self.uri;
      if uri.contains(ALL_STR) {}
    }
  }
}
