use hyper::Uri;

pub trait Rule: Send + Sync {
  fn produce_uri(&self, cmd: &str, args: &[String]) -> Result<Uri, String>;
}

pub static DEFAULT_RULE_KEY: &'static str = "_";

// #[derive(Default)]
// pub struct YouTubeRule;
// impl Rule for YouTubeRule {
//   fn produce_uri(&self, _cmd: &str, args: &Vec<String>) -> Result<Uri, String> {
//     let builder = Uri::builder().scheme("https").authority("youtube.com");

//     let res = match args[..] {
//       [] => builder.path_and_query("/").build(),
//       _ => {
//         let encoded = urlencoding::encode(&args.join(" ")).into_owned();
//         builder
//           .path_and_query(format!("/results?search_query={}", encoded))
//           .build()
//       }
//     };

//     res.map_err(|e| format!("Error producing URI: {}", e))
//   }
// }
