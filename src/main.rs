use clap::{App, Arg};
use http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log;
use pretty_env_logger;
use querystring;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use urlencoding;

pub fn get_request_uid() -> String {
    format!(
        "request-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}

macro_rules! time_request {
    ($req_blk:block) => {{
        let rid = get_request_uid();
        let start = SystemTime::now();
        let res = $req_blk;
        let duration_ms = start.elapsed().unwrap();
        log::trace!("[{}] Completed in {}micros", rid, duration_ms.as_micros());
        res
    }};
}

trait Rule: Send + Sync {
    fn produce_uri(&self, args: &Vec<String>) -> Result<Uri, String>;
}

type Rules = HashMap<String, Box<dyn Rule>>;

#[derive(Default)]
struct GoogleSearchRule;
impl Rule for GoogleSearchRule {
    fn produce_uri(&self, args: &Vec<String>) -> std::result::Result<Uri, String> {
        let encoded_query = urlencoding::encode(&args.join(" ")).into_owned();
        log::debug!(target: "ezproxy::GoogleSearchRule", "Encoding query {} from args {:?}", encoded_query, args);
        Uri::builder()
            .scheme("https")
            .authority("www.google.com")
            .path_and_query(format!("/search?q={}", encoded_query))
            .build()
            .map_err(|e| format!("Error producing URI: {}", e))
    }
}

#[derive(Default)]
struct GmailRule;
impl Rule for GmailRule {
    fn produce_uri(&self, _: &Vec<String>) -> Result<Uri, String> {
        Uri::builder()
            .scheme("https")
            .authority("gmail.com")
            .path_and_query("/")
            .build()
            .map_err(|e| format!("Error producing URI: {}", e))
    }
}

#[derive(Default)]
struct CalendarRule;
impl Rule for CalendarRule {
    fn produce_uri(&self, _: &Vec<String>) -> Result<Uri, String> {
        Uri::builder()
            .scheme("https")
            .authority("calendar.google.com")
            .path_and_query("/")
            .build()
            .map_err(|e| format!("Error producing URI: {}", e))
    }
}

#[derive(Default)]
struct NpmRule;
impl Rule for NpmRule {
    fn produce_uri(&self, args: &Vec<String>) -> Result<Uri, String> {
        let builder = Uri::builder().scheme("https").authority("npmjs.com");

        let res = match args[..] {
            [] => builder.path_and_query("/").build(),
            _ => {
                let encoded = urlencoding::encode(&args.join(" ")).into_owned();
                builder
                    .path_and_query(format!("/search?q={}", encoded))
                    .build()
            }
        };

        res.map_err(|e| format!("Error producing URI: {}", e))
    }
}

#[derive(Debug)]
struct Command {
    name: String,
    args: Vec<String>,
}

#[derive(Clone)]
struct Config {
    config_bytes: Vec<u8>,
}
impl Config {
    pub fn read_from_path(path: &str) -> Result<Self, String> {
        let config_bytes = Self::somehow_read_path(path)?;
        Ok(Self { config_bytes })
    }

    pub fn parse_rules(&self) -> Result<Rules, String> {
        log::warn!(target: "ezproxy::config", "Parsing rules (NOT YET IMPLEMENTED)");
        let mut rules: HashMap<String, Box<dyn Rule>> = HashMap::new();

        // RULESS TODO: Actually parse
        rules.insert("m".into(), Box::new(GmailRule::default()));
        rules.insert("c".into(), Box::new(CalendarRule::default()));
        rules.insert("npm".into(), Box::new(NpmRule::default()));

        Ok(rules)
    }

    fn somehow_read_path(path: &str) -> Result<Vec<u8>, String> {
        log::debug!(target: "ezproxy::config", "Reading config from path={}", path);
        fs::read(path).map_err(|_| "Could not read config path".into())
    }
}

#[derive(Default, Debug)]
struct CommandParser {}
impl CommandParser {
    pub fn parse(&self, uri: &Uri) -> Result<Command, String> {
        log::debug!(target: "ezproxy::command_parser", "Attempt parse {}", uri);

        let query = uri
            .query()
            .map(|qs| querystring::querify(qs))
            .and_then(|params| {
                params.into_iter().find(|param| match param {
                    ("q", _) => true,
                    _ => false,
                })
            })
            .map_or(Err("Could not find query param q=...".to_string()), |p| {
                Ok(p.1.into())
            })
            .map(|q: String| q.replace("+", " "))?;

        let decoded = urlencoding::decode(&query)
            .map(|cow| cow.into_owned())
            .map_err(|_| "Could not decode query".to_owned())?;
        let parts: Vec<String> = decoded.split(" ").map(|s| s.to_string()).collect();
        match &parts[..] {
            [] => Err("Malformed query".to_string()),
            [name] => Ok(Command {
                name: String::from(name),
                args: vec![],
            }),
            p => {
                let name = p[0].to_string();
                let args = p[1..].iter().map(|s| s.to_string()).collect();
                Ok(Command { name, args })
            }
        }
    }
}

struct RulesRegistry {
    rules: Rules,
}

impl RulesRegistry {
    pub fn with_rules(rules: Rules) -> Self {
        Self { rules }
    }

    pub fn get<'a>(&self, rule_name: &str) -> Option<&Box<dyn Rule>> {
        self.rules.get(rule_name)
    }
}

impl Debug for RulesRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RulesRegistry")
            .field("rules", &"[...]")
            .finish()
    }
}

#[derive(Debug)]
struct Redirector {
    cmd_parser: CommandParser,
    rules_registry: RulesRegistry,
}
impl Redirector {
    pub fn with_config(config: &Config) -> Self {
        Self {
            cmd_parser: CommandParser::default(),
            rules_registry: RulesRegistry::with_rules(config.parse_rules().unwrap()),
        }
    }

    pub fn evaluate(&self, uri: &Uri) -> Result<Uri, String> {
        let cmd = self.cmd_parser.parse(&uri)?;
        log::debug!(target: "ezproxy::redirector", "Attempting redirector for {:?}", cmd);
        if let Some(rule) = self.rules_registry.get(&cmd.name) {
            log::debug!(target: "ezproxy::redirector", "Evaluate {} with found rule", cmd.name);
            rule.produce_uri(&cmd.args)
        } else {
            log::debug!(target: "ezproxy::redirector", "No rule found for {}. Using default", cmd.name);
            let mut default_args = vec![];
            default_args.push(cmd.name);
            for arg in cmd.args {
                default_args.push(arg);
            }
            self.do_default(&default_args)
        }
    }

    fn do_default(&self, args: &Vec<String>) -> Result<Uri, String> {
        GoogleSearchRule::default().produce_uri(args)
    }
}

fn uri_from_conn<T>(req: &mut Request<T>) -> Uri {
    req.uri().to_owned()
}

fn somehow_make_response(uri_result: Result<Uri, String>) -> http::Result<Response<Body>> {
    let builder = Response::builder().header("X-EZ-Made-This", "true");

    match uri_result {
        Ok(uri) => builder
            .status(302)
            .header("Location", format!("{}", uri))
            .body(Body::from("")),
        Err(msg) => builder.status(500).body(Body::from(msg)),
    }
}

fn somehow_get_and_validate_args() -> String {
    let matches = App::new("EZProxy")
        .version("0.1.0")
        .about("Turns your address bar into a CLI")
        .arg(
            Arg::new("config")
                .takes_value(true)
                .required(true)
                .help("Path to configuration"),
        )
        .get_matches();

    matches
        .value_of("config")
        .expect("Must supply path")
        .to_string()
}

#[derive(Clone)]
struct AppContext {
    redirector: Arc<Redirector>,
}

async fn handle(context: AppContext, mut req: Request<Body>) -> http::Result<Response<Body>> {
    time_request!({
        let eval_result = match context.redirector.evaluate(&uri_from_conn(&mut req)) {
            Ok(uri) => {
                log::info!(target: "ezproxy::handle", "Returning uri {}", uri);
                Ok(uri)
            }
            Err(e) => {
                log::error!(target: "ezproxy::handle", "Error evaluating request: {}", e);
                Err(e)
            }
        };
        somehow_make_response(eval_result)
    })
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
    log::info!(target: "ezproxy::boot", "Starting on {}", addr);

    let path = somehow_get_and_validate_args();
    let config = Config::read_from_path(&path).unwrap();
    let context = AppContext {
        redirector: Arc::new(Redirector::with_config(&config)),
    };
    let make_service = make_service_fn(move |_conn| {
        let context = context.clone();
        let service = service_fn(move |req| handle(context.clone(), req));
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
