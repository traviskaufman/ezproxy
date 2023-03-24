use crate::rules::Rule;
use hyper::Uri;
use lazy_static::lazy_static;
use log;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// TODO:
/// - Support comments
/// - Support things like default URL vs. having ARGS (see commented-out YT)
/// - Maybe rule needs to have produce_default() and produce_args()?
pub fn parse_rules_from<P: AsRef<Path>>(path: P) -> HashMap<String, Box<dyn Rule>> {
  lazy_static! {
    static ref RULE_RE: Regex = Regex::new(r#"^(.+)\s=\s(.+)"#).unwrap();
  }
  let data = fs::read_to_string(path).unwrap();
  let config_rules = data.trim().split("\n").map(|line| {
    let ex = format!("Malformed config URL {}: expected (kw) = (url)", line);
    let captures = RULE_RE.captures(line).expect(&ex);
    ConfigRule::new(&captures[1], &captures[2])
  });
  let mut rules: HashMap<String, Box<dyn Rule>> = HashMap::new();
  for cfg_rule in config_rules {
    log::info!("Insert {}", cfg_rule.kw());
    rules.insert(cfg_rule.kw().to_string(), Box::new(cfg_rule));
  }
  rules
}

#[derive(Debug)]
pub struct ConfigRule {
  kw: String,
  uri: String,
}

impl ConfigRule {
  pub fn new<K: Into<String>, U: Into<String>>(kw: K, uri: U) -> Self {
    Self {
      kw: kw.into(),
      uri: uri.into(),
    }
  }

  pub fn kw(&self) -> &str {
    &self.kw
  }
}

impl fmt::Display for ConfigRule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "ConfigRule[kw={}, uri={}]", self.kw, self.uri)
  }
}

impl Rule for ConfigRule {
  fn produce_uri(&self, cmd: &str, args: &[String]) -> Result<Uri, String> {
    const ARGS_STR: &str = "{ARGS}";
    const ALL_STR: &str = "{ALL}";

    let uri_str = if self.uri.contains(ALL_STR) {
      let all_str = format!("{} {}", cmd, args.join(" "));
      self.uri.replace(ALL_STR, &urlencoding::encode(&all_str))
    } else if self.uri.contains(ARGS_STR) {
      self
        .uri
        .replace(ARGS_STR, &urlencoding::encode(&args.join(" ")))
    } else {
      self.uri.clone()
    };

    log::debug!("Produce URI {}", uri_str);
    uri_str
      .parse::<Uri>()
      .map_err(|e| format!("URI Parse error for {}: {}", uri_str, e).into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new_config_rule() {
    let config_rule = ConfigRule::new("test_kw", "test_uri");
    assert_eq!(config_rule.kw(), "test_kw");
  }

  #[test]
  fn produce_uri_all() {
    let config_rule = ConfigRule::new("test_kw", "http://example.com/{ALL}");
    let cmd = "test_cmd";
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    let result = config_rule.produce_uri(cmd, &args);
    assert!(result.is_ok());
    let uri = result.unwrap();
    assert_eq!(uri.to_string(), "http://example.com/test_cmd%20arg1%20arg2");
  }

  #[test]
  fn produce_uri_args() {
    let config_rule = ConfigRule::new("test_kw", "http://example.com/{ARGS}");
    let cmd = "test_cmd";
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    let result = config_rule.produce_uri(cmd, &args);
    assert!(result.is_ok());
    let uri = result.unwrap();
    assert_eq!(uri.to_string(), "http://example.com/arg1%20arg2");
  }

  #[test]
  fn produce_uri_no_replace() {
    let config_rule = ConfigRule::new("test_kw", "http://example.com/");
    let cmd = "test_cmd";
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    let result = config_rule.produce_uri(cmd, &args);
    assert!(result.is_ok());
    let uri = result.unwrap();
    assert_eq!(uri.to_string(), "http://example.com/");
  }
}
