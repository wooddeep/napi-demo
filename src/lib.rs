use futures::prelude::*;
use napi::bindgen_prelude::*;
use tokio::fs;
//use napi::tokio::{self, fs};
//#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

/// async fn, require `async` feature enabled.
/// [dependencies]
/// napi = {version="2", features=["async"]}
#[napi]
async fn read_file_async(path: String) -> Result<Buffer> {
  fs::read(path)
    .map(|r| match r {
      Ok(content) => Ok(content.into()),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("failed to read file, {}", e),
      )),
    })
    .await
}



#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}

