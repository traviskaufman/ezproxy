use ezproxy::rules::*;
use http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use log;
use pretty_env_logger;
use querystring;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Debug;
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

#[derive(Debug)]
struct Command {
    name: String,
    args: Vec<String>,
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

static DEFAULT_RULE_KEY: &'static str = "_";

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

    pub fn default() -> Self {
        let mut rules: HashMap<String, Box<dyn Rule>> = HashMap::new();
        rules.insert("m".to_owned(), Box::new(GmailRule::default()));
        rules.insert("c".to_owned(), Box::new(CalendarRule::default()));
        rules.insert("yt".to_owned(), Box::new(YouTubeRule::default()));
        rules.insert("npm".to_owned(), Box::new(NpmRule::default()));
        rules.insert(
            DEFAULT_RULE_KEY.to_owned(),
            Box::new(GoogleSearchRule::default()),
        );
        Redirector::with_rules(rules)
    }

    pub fn evaluate(&self, uri: &Uri) -> Result<Uri, String> {
        let cmd = self.cmd_parser.parse(&uri)?;
        log::debug!(target: "ezproxy::redirector", "Attempting redirector for {:?}", cmd);
        if let Some(rule) = self.rules.get(&cmd.name) {
            rule.produce_uri(&cmd.args)
        } else if let Some(default_rule) = self.rules.get(DEFAULT_RULE_KEY) {
            log::debug!(target: "ezproxy::redirector", "No rule found for {}. Using default", cmd.name);
            let mut default_args = vec![];
            default_args.push(cmd.name);
            for arg in cmd.args {
                default_args.push(arg)
            }
            default_rule.produce_uri(&default_args)
        } else {
            Err(format!(
                "Could not find rule for cmd {}, and no default given",
                cmd.name
            ))
        }
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

    let context = AppContext {
        redirector: Arc::new(Redirector::default()),
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
