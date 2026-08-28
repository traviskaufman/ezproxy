use assert_fs::prelude::*;
use duct;
use hyper::Client;
use scopeguard;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time;
use tokio;

fn assert_free_port() -> u16 {
    (1025..65535)
        .find(
            |port| match TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], *port))) {
                Ok(_) => true,
                Err(_) => false,
            },
        )
        .expect("No free available ports!")
}

fn wait_for_listener(port: u16) {
    let deadline = time::Instant::now() + time::Duration::from_secs(120);
    while TcpStream::connect(SocketAddr::from(([127, 0, 0, 1], port))).is_err() {
        assert!(
            time::Instant::now() < deadline,
            "server never started listening"
        );
        thread::sleep(time::Duration::from_millis(100));
    }
}

#[tokio::test]
async fn test_ezproxy() {
    static CONFIG: &'static str = r#"
m = https://gmail.com/
npm = https://npmjs.com/search?q={ARGS}
_ = https://www.google.com/search?q={ALL}
  "#;

    let config_file = assert_fs::NamedTempFile::new("config.txt").unwrap();
    let config_file = scopeguard::guard(config_file, |f| {
        f.close().unwrap();
    });
    config_file.write_str(CONFIG).unwrap();

    let port = assert_free_port();
    let handle = duct::cmd!(
        "cargo",
        "run",
        "--release",
        "--",
        "--port",
        format!("{}", port),
        config_file.path(),
    )
    .start()
    .unwrap();

    wait_for_listener(port);

    let client = Client::new();
    let uri = format!("http://localhost:{}/?q=m", port).parse().unwrap();
    let resp = client.get(uri).await.unwrap();

    assert_eq!(resp.status(), 302);
    assert_eq!(
        resp.headers()
            .get("Location")
            .expect("Expected Location Header"),
        "https://gmail.com/"
    );

    let uri = format!("http://localhost:{}/?q=npm%20file%20finder", port)
        .parse()
        .unwrap();
    let resp = client.get(uri).await.unwrap();
    assert_eq!(
        resp.headers()
            .get("Location")
            .expect("Expected Location Header"),
        "https://npmjs.com/search?q=file%20finder"
    );

    let uri = format!("http://localhost:{}/?q=best%20restaurants%20nyc", port)
        .parse()
        .unwrap();
    let resp = client.get(uri).await.unwrap();
    assert_eq!(
        resp.headers()
            .get("Location")
            .expect("Expected Location Header"),
        "https://www.google.com/search?q=best%20restaurants%20nyc"
    );

    handle.kill().unwrap();
}
