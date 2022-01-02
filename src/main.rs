use clap::{App, Arg};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log;
use pretty_env_logger;
use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::SystemTime;

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
        log::trace!("[{}] Completed in {}ms", rid, duration_ms.as_millis());
        res
    }};
}

trait Rule {
    fn produce_uri(&self, args: &Vec<String>) -> Result<SomeUriType, String>;
}

struct Command {
    name: String,
    args: Vec<String>,
}

struct Config {
    config_bytes: Vec<u8>,
}
impl Config {
    pub fn read_from_path(path: &str) -> Result<Self, String> {
        let config_bytes = Self::somehow_read_path(path)?;
        Ok(Self { config_bytes })
    }

    // TODO: Figure out the rust way to do this
    pub fn parse_rules(&self) -> Result<Vec<Box<dyn Rule>>, String> {
        todo!();
    }

    fn somehow_read_path(path: &str) -> Result<Vec<u8>, String> {
        todo!();
    }
}

#[derive(Default, Debug)]
struct CommandParser {}
impl CommandParser {
    pub fn parse(&self, uri: &SomeUriType) -> Result<Command, String> {
        todo!()
    }
}

type Rules = Vec<Box<dyn Rule>>;

struct RulesRegistry {
    rules: Rules,
}

impl Debug for RulesRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RulesRegistry")
            .field("rules", &"[...]")
            .finish()
    }
}

impl RulesRegistry {
    pub fn with_rules(rules: Rules) -> Self {
        Self { rules }
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

    pub fn evaluate(&self, uri: &SomeUriType) -> Result<SomeUriType, String> {
        let cmd = self.cmd_parser.parse(&uri)?;
        if let Some(rule) = self.rules_registry.get(cmd.name) {
            rule.produce_uri(&cmd.args)
        } else {
            self.do_default(&cmd.args)
        }
    }

    fn do_default(&self, args: &Vec<String>) -> Result<SomeUriType, Infallible> {
        GoogleSearchRule::default()
            .produce_uri(args)
            .expect("Default rules must *always* produce a URI")
    }
}

fn somehow_read_request_uri() {}

fn somehow_make_response(uri: SomeUriType) {}

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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 5050));
    log::info!("[boot] Starting on {}", addr);

    let config = Config::read_from_path(&somehow_get_and_validate_args()).unwrap();
    let redirector = Redirector::with_config(&config);
    let make_service = make_service_fn(|_conn| async move {
        let res = time_request!({
            redirector
                .evaluate(&somehow_read_request_uri())
                .map(|uri| somehow_make_response(uri))
        })
        .map_err(|_| todo!());
        Ok::<_, Infallible>(res)
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
