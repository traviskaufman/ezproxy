use hyper::Uri;

pub trait Rule: Send + Sync {
  fn produce_uri(&self, cmd: &str, args: &Vec<String>) -> Result<Uri, String>;
}

#[derive(Default)]
pub struct GoogleSearchRule;
impl Rule for GoogleSearchRule {
  fn produce_uri(&self, cmd: &str, args: &Vec<String>) -> std::result::Result<Uri, String> {
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
pub struct GmailRule;
impl Rule for GmailRule {
  fn produce_uri(&self, _cmd: &str, _args: &Vec<String>) -> Result<Uri, String> {
    Uri::builder()
      .scheme("https")
      .authority("gmail.com")
      .path_and_query("/")
      .build()
      .map_err(|e| format!("Error producing URI: {}", e))
  }
}

/// What I want:
/// #[rule("https://calendar.google.com/")]
/// struct CalendarRule

#[derive(Default)]
pub struct CalendarRule;
impl Rule for CalendarRule {
  fn produce_uri(&self, _cmd: &str, _args: &Vec<String>) -> Result<Uri, String> {
    Uri::builder()
      .scheme("https")
      .authority("calendar.google.com")
      .path_and_query("/")
      .build()
      .map_err(|e| format!("Error producing URI: {}", e))
  }
}

#[derive(Default)]
pub struct NpmRule;
impl Rule for NpmRule {
  fn produce_uri(&self, _cmd: &str, args: &Vec<String>) -> Result<Uri, String> {
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

#[derive(Default)]
pub struct YouTubeRule;
impl Rule for YouTubeRule {
  fn produce_uri(&self, _cmd: &str, args: &Vec<String>) -> Result<Uri, String> {
    let builder = Uri::builder().scheme("https").authority("youtube.com");

    let res = match args[..] {
      [] => builder.path_and_query("/").build(),
      _ => {
        let encoded = urlencoding::encode(&args.join(" ")).into_owned();
        builder
          .path_and_query(format!("/results?search_query={}", encoded))
          .build()
      }
    };

    res.map_err(|e| format!("Error producing URI: {}", e))
  }
}
