extern crate napi_build;
extern crate cc;
use cc::Build;

fn main() {
  napi_build::setup();
  // // 编译 C 代码并生成库文件
  // Build::new()
  //     .file("src/ipc.c")
  //     .compile("ipc");
}



