use crate::rules::Rule;
use hyper::Uri;
use lazy_static::lazy_static;
use log;
use regex::Regex;
use std::collections::HashMap;
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

/// TODO:
/// - Maybe have it parse up front to speed up evals and avoid regexes on every run
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
impl Rule for ConfigRule {
  // Alter args such that the cmd name is the first arg
  fn produce_uri(&self, cmd: &str, args: &Vec<String>) -> Result<Uri, String> {
    static ARGS_STR: &'static str = "{ARGS}";
    static ALL_STR: &'static str = "{ALL}";
    let uri = self.uri.clone();
    let args_str = args.join(" ");

    let uri_str = if uri.contains(ALL_STR) {
      let all_str = format!("{} {}", cmd, args_str);
      uri.replace(ALL_STR, &urlencoding::encode(&all_str))
    } else if uri.contains(ARGS_STR) {
      uri.replace(ARGS_STR, &urlencoding::encode(&args_str))
    } else {
      uri
    };
    log::debug!("Produce URI {}", uri_str);

    let uri = uri_str
      .parse::<Uri>()
      .map_err(|e| String::from(format!("URI Parse error for {}: {}", uri_str, e)))?;
    Ok(uri)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_static() -> Result<(), String> {
    let rule = ConfigRule::new("m", "https://gmail.com/");
    let args = vec![];
    assert_eq!(
      format!("{}", rule.produce_uri("m", &args)?),
      "https://gmail.com/"
    );
    Ok(())
  }

  #[test]
  fn test_args() -> Result<(), String> {
    let rule = ConfigRule::new("npm", "https://npmjs.com/?q={ARGS}");
    let args = vec![String::from("parse"), String::from("file")];
    let uri = rule.produce_uri("npm", &args)?;
    assert_eq!(format!("{}", uri), "https://npmjs.com/?q=parse%20file");
    Ok(())
  }

  #[test]
  fn test_all() -> Result<(), String> {
    let rule = ConfigRule::new("google", "https://google.com/search?q={ALL}");
    let args = vec![String::from("parse"), String::from("file")];
    let uri = rule.produce_uri("rustlang", &args)?;
    assert_eq!(
      format!("{}", uri),
      "https://google.com/search?q=rustlang%20parse%20file"
    );
    Ok(())
  }
}
