use crate::rules::Rule;
use hyper::Uri;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// TODO:
/// - Support comments
/// - Support things like default URL vs. having ARGS
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
    let mut uri = self.uri.clone();
    let args_str = args.join(" ");
    let args_str = urlencoding::encode(&args_str);

    if uri.contains(ALL_STR) {
      let all_str = format!("{} {}", cmd, args_str);
      let all_str = urlencoding::encode(&all_str);
      uri = uri.replace(ALL_STR, &all_str);
    } else if uri.contains(ARGS_STR) {
      uri = uri.replace(ARGS_STR, &args_str);
    }
    let uri = uri
      .parse::<Uri>()
      .map_err(|_| String::from("URI Parse error"))?;
    Ok(uri)
  }
}
