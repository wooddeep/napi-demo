use futures::prelude::*;
use napi::bindgen_prelude::*;
use std::process;
use tokio::fs;
//use napi::tokio::{self, fs};
//#![deny(clippy::all)]

use futures::StreamExt as _;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

use parity_tokio_ipc::{Endpoint, SecurityAttributes};



#[macro_use]
extern crate napi_derive;

// https://tokio.rs/tokio/tutorial/channels
// https://napi.rs/docs/concepts/async-task
// https://napi.rs/docs/concepts/async-fn
// https://napi.rs/docs/compat-mode/concepts/tokio

// https://docs.rs/interprocess/latest/interprocess/os/windows/named_pipe/enum.PipeDirection.html

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

#[napi]
pub fn show() -> u32 {
  process::id()
}

#[napi]
pub fn start() {
  run_server(process::id().to_string());
}

async fn run_server(path: String) {
  let mut endpoint = Endpoint::new(path);
  endpoint.set_security_attributes(SecurityAttributes::allow_everyone_create().unwrap());

  let incoming = endpoint.incoming().expect("failed to open new socket");
  futures::pin_mut!(incoming);

  while let Some(result) = incoming.next().await {
    match result {
      Ok(stream) => {
        let (mut reader, mut writer) = split(stream);

        tokio::spawn(async move {
          loop {
            let mut buf = [0u8; 4];
            let pong_buf = b"pong";
            if let Err(_) = reader.read_exact(&mut buf).await {
              println!("Closing socket");
              break;
            }
            if let Ok("ping") = std::str::from_utf8(&buf[..]) {
              println!("RECIEVED: PING");
              writer
                .write_all(pong_buf)
                .await
                .expect("unable to write to socket");
              println!("SEND: PONG");
            }
          }
        });
      }
      _ => unreachable!("ideally"),
    }
  }
}

#[napi]
pub fn start_client() {
  start_client()
}

async fn run_start_client() {
  let path = std::env::args().nth(1).expect("Run it with server path to connect as argument");

  let mut client = Endpoint::connect(&path).await
      .expect("Failed to connect client.");

  loop {
    let mut buf = [0u8; 4];
    println!("SEND: PING");
    client.write_all(b"ping").await.expect("Unable to write message to client");
    client.read_exact(&mut buf[..]).await.expect("Unable to read buffer");
    if let Ok("pong") = std::str::from_utf8(&buf[..]) {
      println!("RECEIVED: PONG");
    } else {
      break;
    }

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
  }
}