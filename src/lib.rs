mod ipc;

use futures::prelude::*;
use napi::bindgen_prelude::*;
use std::process;

use std::sync::mpsc::channel;
use std::thread;
use tokio::task;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;
use std::path::Path;

use std::io::Error;

use std::ptr::{null_mut};
use winapi::ctypes::c_ulong;


use napi::{JsFunction, Result};
use std::{time};
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

#[macro_use]
extern crate napi_derive;

// https://tokio.rs/tokio/tutorial/channels
// https://napi.rs/docs/concepts/async-task
// https://napi.rs/docs/concepts/async-fn
// https://napi.rs/docs/compat-mode/concepts/tokio

// https://docs.rs/interprocess/latest/interprocess/os/windows/named_pipe/enum.PipeDirection.html

// 主进程初始化meta data
#[derive(Deserialize, Serialize)]
struct ProcReadStat {
    stat: bool,
    pid: u32,
}

#[derive(Deserialize, Serialize)]
struct MsgMeta {
    msg_id: u32,
    slot_id: u32,
    prev_slot: u32,
    next_slot: u32,
    stat_list: Vec<ProcReadStat>,
}

#[derive(Deserialize, Serialize)]
struct MsgRingList {
    head: u32,
    tail: u32,
    msg_list: Vec<MsgMeta>,
}

struct ShareMemMeta {
    pid: u32,
}

// serialize one message
fn msg_meta_serial() -> Vec<u8> {
    let mm = MsgMeta {
        msg_id: 0,
        slot_id: 1,
        prev_slot: 10,
        next_slot: 11,
        stat_list: vec![ProcReadStat {
            stat: true,
            pid: 100,
        }, ProcReadStat {
            stat: false,
            pid: 101,
        }],
    };

    // Or just to a buffer
    let bytes = bincode::serialize(&mm).unwrap();
    println!("{:?}", bytes);

    // // Encode to something implementing `Write`
    // let mut f = File::create("/tmp/output.bin").unwrap();
    // bincode::serialize_into(&mut f, &a).unwrap();
    return bytes;
}

// ipc lock: https://docs.rs/proc-lock/latest/proc_lock/

// 根据子进程数目, 初始化共享内存区
// | child process number(n)| process_0 id | ...... | process_n-1 id|
#[derive(Deserialize, Serialize)]
struct ProcInfo {
    process_id: u32,
    occupied: bool,
}

#[derive(Deserialize, Serialize)]
struct ProcTable {
    proc_num: u32,
    proc_list: Vec<ProcInfo>,
}

#[napi]
pub fn init_proc_table(n: u32) {

    // let shmem_flink = Path::new("basic_mapping");
    // let mut proc_table = ProcTable { proc_num: n, proc_list: vec![] };
    // for i in 0..n - 1 {
    //     let proc_info = ProcInfo { process_id: 100, occupied: true };
    //     proc_table.proc_list.push(proc_info)
    // }
    //
    // let bytes = bincode::serialize(&proc_table).unwrap();
    // println!("{:?}", bytes);
    //
    // // Get pointer to the shared memory
    // let mut raw_ptr = shmem.as_ptr();
    //
    // // WARNING: This is prone to race conditions as no sync/locking is used
    // unsafe {
    //     for i in 0..bytes.len() {
    //         *(raw_ptr.offset(i as isize)) = bytes[i];
    //     }
    // }

    println!("share memory create done!");
}

#[napi]
pub fn init_proc_info(brothers: u32, index: u32) {
    // let shmem_flink = Path::new("basic_mapping");
    //
    // let mut proc_table = ProcTable { proc_num: brothers, proc_list: vec![] };
    // for i in 0..brothers - 1 {
    //     let proc_info = ProcInfo { process_id: 100, occupied: true };
    //     proc_table.proc_list.push(proc_info)
    // }
    //
    // let mut bytes = bincode::serialize(&proc_table).unwrap();
    // println!("0: {:?}", bytes);
    //
    // let shmem =
    //
    // // Get pointer to the shared memory
    // let mut raw_ptr = shmem.as_ptr();
    //
    // unsafe {
    //     for i in 0..bytes.len() {
    //         bytes[i] = *(raw_ptr.offset(i as isize));
    //     }
    // }
    // println!("1: {:?}", bytes);
    //
    // let proc_table: ProcTable = bincode::deserialize(&*bytes).unwrap();
    // println!("## process number: {}", proc_table.proc_num);
    // for i in 0..proc_table.proc_num {
    //     let proc_info = proc_table.proc_list.get(i as usize).unwrap();
    //     println!("## process id: {}, occupied: {}", proc_info.process_id, proc_info.process_id);
    // }
}

use winapi::shared::minwindef::{LPVOID};
use winapi::um::winnt::HANDLE;
use std::ptr;

#[cfg(target_os = "windows")]
static mut MAP_DESC: (LPVOID, HANDLE) = (ptr::null_mut(), ptr::null_mut());
#[cfg(target_os = "windows")]
static mut NOTIFY_SEMA: HANDLE = ptr::null_mut();
static  SEMA_NAME: &str = "test";

#[napi]
pub async fn test_sema_release() {
    task::spawn_blocking(move || {
        let semaphore = ipc::sema_open(SEMA_NAME);
        ipc::sema_release(semaphore);
        ipc::sema_close(semaphore);
    }).await.unwrap();
}

#[napi]
pub async fn test_sema_require() {
    task::spawn_blocking(move || {
        let semaphore = ipc::sema_open(SEMA_NAME);
        ipc::sema_require(semaphore);
        ipc::sema_close(semaphore);
    }).await.unwrap();
}


#[napi]
pub fn show() -> u32 {
    process::id()
}


#[napi]
pub async fn master_init() {
    unsafe {
        MAP_DESC = ipc::shm_init();
        let buffer = b"Hello, Rust";
        ipc::do_shm_write(MAP_DESC.0, buffer);
        NOTIFY_SEMA = ipc::sema_create(SEMA_NAME);
    }
}

#[napi]
pub fn worker_init() {
    thread::spawn(|| {
        unsafe {
            MAP_DESC = ipc::shm_init();
            ipc::do_shm_read(MAP_DESC.0);
            NOTIFY_SEMA = ipc::sema_open(SEMA_NAME);
        }
    });
}

#[napi]
pub fn process_exit() {
    #[cfg(target_os = "windows")] unsafe {
        //UnmapViewOfFile(MAP_DESC.0);
        //CloseHandle(MAP_DESC.1);
    }
}

// index: worker process index: from 0
#[napi]
pub fn send_data(index: u32, data: Buffer, n: u32) {

    let buffer = unsafe {
        let slice = std::slice::from_raw_parts(data.as_ptr() as *const u8, n as usize);
        std::str::from_utf8_unchecked(slice)
    };

    println!("data: {}", buffer);
}


#[napi]
pub fn call_threadsafe_function(callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<u32, ErrorStrategy::CalleeHandled> = callback
        .create_threadsafe_function(0, |ctx| {
            ctx.env.create_uint32(ctx.value + 1).map(|v| vec![v])
        })?;

    let one_second = time::Duration::from_secs(1);

    let tsfn = tsfn.clone();
    thread::spawn(move || {
        loop {
            tsfn.call(Ok(0), ThreadsafeFunctionCallMode::NonBlocking);
            thread::sleep(one_second);
        }
    });

    Ok(())
}

use napi::{Env};

fn clearup(env: Env) {
    println!("#shut down!!")
}

#[napi]
pub fn init(mut env: Env) -> Result<()> {
    env.add_env_cleanup_hook(env, clearup);
    //env.add_async_cleanup_hook(env, clearup);
    Ok(())
}