use log;
use std::time::SystemTime;

#[macro_export]
macro_rules! time_request {
  ( req_blk:block ) => {
    let rid = gen_request_uid();
    let start = SystemTime::now();
    let res = $req_blk;
    let duration_ms = start.elapsed().unwrap();
    log::trace!("[{}] Completed in {}", rid, duration_ms);
    res
  };
}

pub fn get_request_uid() -> String {
  format!(
    "request-{}",
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs()
  )
}
