use clap::Parser;
use ezproxy::config;
use ezproxy::rules::*;
use http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

macro_rules! time_request {
    ($req_blk:block) => {{
        let start = SystemTime::now();
        let res = $req_blk;
        log::trace!(
            "[request-{}] Completed in {}micros",
            start
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            start.elapsed().unwrap().as_micros()
        );
        res
    }};
}

#[derive(Debug)]
struct Command {
    name: String,
    args: Vec<String>,
}

#[derive(Default, Debug)]
struct CommandParser {}
impl CommandParser {
    pub fn parse(&self, uri: &Uri) -> Result<Command, String> {
        log::debug!(target: "ezproxy::command_parser", "Attempt parse {uri}");

        let query = uri
            .query()
            .map(|qs| querystring::querify(qs))
            .and_then(|params| params.into_iter().find(|param| matches!(param, ("q", _))))
            .ok_or("Could not find query param q=...")?
            .1
            .replace('+', " ");

        let decoded =
            urlencoding::decode(&query).map_err(|_| "Could not decode query".to_owned())?;
        let mut parts = decoded.split(' ').map(String::from);
        let name = parts.next().unwrap_or_default();
        let args = parts.collect();
        Ok(Command { name, args })
    }
}

struct Redirector {
    cmd_parser: CommandParser,
    rules: HashMap<String, Box<dyn Rule>>,
}
impl Redirector {
    pub fn with_rules(rules: HashMap<String, Box<dyn Rule>>) -> Self {
        Self {
            rules,
            cmd_parser: CommandParser::default(),
        }
    }

    pub fn with_config<P: AsRef<Path>>(config_path: P) -> Self {
        let rules = config::parse_rules_from(config_path);
        Redirector::with_rules(rules)
    }

    pub fn evaluate(&self, uri: &Uri) -> Result<Uri, String> {
        let cmd = self.cmd_parser.parse(uri)?;
        log::debug!(target: "ezproxy::redirector", "Attempting redirector for {cmd:?}");
        if let Some(rule) = self.rules.get(&cmd.name) {
            rule.produce_uri(&cmd.name, &cmd.args)
        } else if let Some(default_rule) = self.rules.get(DEFAULT_RULE_KEY) {
            log::debug!(target: "ezproxy::redirector", "No rule found for {}. Using default", cmd.name);
            default_rule.produce_uri(&cmd.name, &cmd.args)
        } else {
            Err(format!(
                "Could not find rule for cmd {}, and no default given",
                cmd.name
            ))
        }
    }
}

fn somehow_make_response(uri_result: Result<Uri, String>) -> http::Result<Response<Body>> {
    let builder = Response::builder().header("X-EZ-Made-This", "true");

    match uri_result {
        Ok(uri) => builder
            .status(302)
            .header("Location", format!("{uri}"))
            .body(Body::from("")),
        Err(msg) => builder.status(500).body(Body::from(msg)),
    }
}

#[derive(Clone)]
struct AppContext {
    redirector: Arc<Redirector>,
}

async fn handle(context: AppContext, req: Request<Body>) -> http::Result<Response<Body>> {
    time_request!({
        let eval_result = match context.redirector.evaluate(req.uri()) {
            Ok(uri) => {
                log::info!(target: "ezproxy::handle", "Returning uri {uri}");
                Ok(uri)
            }
            Err(e) => {
                log::error!(target: "ezproxy::handle", "Error evaluating request: {e}");
                Err(e)
            }
        };
        somehow_make_response(eval_result)
    })
}

/// Keyboard shortcuts for your address bar
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the config file used to specify shortcuts. See example-configs/simple.txt for a starter config.
    #[clap(value_parser, value_name = "FILE")]
    config: PathBuf,

    /// Port which ezproxy will run on
    #[clap(short, long, value_parser, default_value_t = 5050)]
    port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    log::info!(target: "ezproxy::boot", "Starting on {addr}");

    let context = AppContext {
        redirector: Arc::new(Redirector::with_config(&args.config)),
    };
    let make_service = make_service_fn(move |_conn| {
        let context = context.clone();
        let service = service_fn(move |req| handle(context.clone(), req));
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("Server error: {e}");
    }
}
