use napi::{bindgen_prelude::*, JsNull, JsString, JsUnknown, threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode}};
use futures::prelude::*;
use tokio::task;

use napi::{JsFunction, Result};
use napi::bindgen_prelude::*;
use napi::{Env};

use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;

use std::sync::mpsc::channel;
use std::ptr::{null_mut};
use std::path::Path;
use std::io::Error;
use std::process;
use std::thread;
use std::{time};
use std::collections::HashMap;
use std::ptr;

use winapi::shared::minwindef::{LPVOID};
use winapi::um::winnt::HANDLE;
use winapi::ctypes::c_ulong;

mod ipc;

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


// one process write, one process read, one semaphore for protect
#[derive(Deserialize, Serialize)]
struct ShareMemCell {
    head_index: i32,
    tail_index: i32,
    msg_list: Vec<Vec<u8>>, // for message restore, 100 message and 200 bytes per message!
}

#[cfg(target_os = "windows")]
static mut MAP_DESC: (LPVOID, HANDLE) = (ptr::null_mut(), ptr::null_mut());
#[cfg(target_os = "windows")]
//static mut NOTIFY_SEMA: HANDLE = ptr::null_mut();
static SEMA_NAME: &str = "test";
static mut NOTIFY_SEMA_ARR: Vec<HANDLE> = Vec::new();
static mut NOTIFY_SEMA_MAP: Vec<Vec<HANDLE>> = Vec::new();

//  100 message per cell
static MAX_MSG_CNT_PER_CELL: u32 = 100;
//  200 bytes per message!
static MAX_MSG_LEN: u32 = 200;
static mut MAX_WORKER_NUM: u32 = 0;
// sizeof(head_index) + sizeof(tail_index)
static MSG_CELL_META_SIZE: u32 = 8;
static MSG_CELL_SIZE: u32 = MSG_CELL_META_SIZE + MAX_MSG_CNT_PER_CELL * MAX_MSG_LEN;
static mut WORKER_INDEX: u32 = 0;

#[napi]
pub async fn test_sema_release() {
    task::spawn_blocking(move || {
        unsafe {
            ipc::sema_release(NOTIFY_SEMA_ARR[WORKER_INDEX as usize]);
        }
    }).await.unwrap();
}

#[napi]
pub async fn test_sema_require() {
    task::spawn_blocking(move || {
        unsafe {
            ipc::sema_require(NOTIFY_SEMA_ARR[WORKER_INDEX as usize]);
        }
    }).await.unwrap();
}

fn get_shm_u32(offset: u32) -> u32 {
    unsafe {
        let head_box = ipc::do_shm_read_buf(MAP_DESC.0, offset, 4);
        let mut head_buf: [u8; 4] = [0; 4];
        head_buf.copy_from_slice(&head_box[..4]);
        let head_index = u32::from_be_bytes(head_buf);
        return head_index;
    }
}

fn set_shm_u32(udata: u32, offset: u32) {
    unsafe {
        let buff: [u8; 4] = udata.to_be_bytes();
        ipc::do_shm_write(MAP_DESC.0, offset, &buff);
    }
}

fn msg_offset(writer_index: u32, reader_index: u32, msg_index: u32) -> u32 {
    unsafe {
        let offset = (writer_index * MAX_WORKER_NUM + reader_index) * MSG_CELL_SIZE +
            MSG_CELL_META_SIZE + msg_index * MAX_MSG_LEN; // write to tail
        offset
    }
}

#[napi]
pub async fn test_shm_write(input: String) {
    unsafe {
        let i = WORKER_INDEX; // writer index

        for j in 0..MAX_WORKER_NUM { // reader index
            if i == j {
                continue;
            }

            ipc::sema_require(NOTIFY_SEMA_MAP[i as usize][j as usize]);

            // get the head & tail index
            let mut tail_index = get_shm_u32(4);
            let buffer = input.as_bytes();
            let offset = msg_offset(i, j, tail_index); // write to tail

            ipc::do_shm_write(MAP_DESC.0, offset, buffer);

            tail_index = tail_index + 1; // tail inc

            // write back the tail index
            set_shm_u32(tail_index, 4);

            ipc::sema_release(NOTIFY_SEMA_MAP[i as usize][j as usize]);
        }
    }
}

use string_builder::Builder;

#[napi]
pub fn test_shm_read() -> String {
    let out =
        unsafe {
            let mut builder = Builder::default();
            builder.append("[");

            let i = WORKER_INDEX; // reader index

            for j in 0..MAX_WORKER_NUM { // writer index
                if i == j {
                    continue;
                }

                // get the head & tail index
                let mut head_index = get_shm_u32(0);
                let mut tail_index = get_shm_u32(4);

                for k in head_index..tail_index {
                    let offset = msg_offset(j, i, k); // read from head
                    let data = ipc::do_shm_read_str(MAP_DESC.0, offset, MAX_MSG_LEN); // read
                    let empty: [u8; 200] = [0; 200]; // msg length: 200 byts equal to MAX_MSG_LEN
                    ipc::do_shm_write(MAP_DESC.0, offset, &empty); // clear
                    builder.append(data);
                    builder.append(",");
                }

                // write back head & tail index
                head_index = 0;
                tail_index = 0;

                set_shm_u32(head_index, 0);
                set_shm_u32(tail_index, 4);
            }

            if builder.len() < 2 {
                //builder
            }

            builder.append("]");
            builder.string().unwrap()
        };
    return out;
}

#[napi]
pub fn show() -> u32 {
    process::id()
}

enum sema_oper {
    CREATE,
    OPEN,
}

fn init_sema_map(worker_num: u32, oper: sema_oper) {
    unsafe {
        for i in 0..worker_num {
            let mut row: Vec<HANDLE> = Vec::new();
            for j in 0..worker_num {
                if i != j {
                    let sema = match oper {
                        sema_oper::CREATE => {
                            ipc::sema_create(&format!("{}-{}-{}", SEMA_NAME, i, j))
                        }
                        sema_oper::OPEN => {
                            ipc::sema_open(&format!("{}-{}-{}", SEMA_NAME, i, j))
                        }
                    };
                    row.push(sema);
                } else {
                    row.push(ptr::null_mut());
                }
            }
            NOTIFY_SEMA_MAP.push(row);
        }
    }
}

#[napi]
pub async fn master_init(worker_num: u32) {
    unsafe {
        MAX_WORKER_NUM = worker_num;
        let shm_size = MAX_WORKER_NUM * MAX_WORKER_NUM * MSG_CELL_SIZE;
        MAP_DESC = ipc::shm_init(shm_size);
        init_sema_map(worker_num, sema_oper::CREATE);
    }
}

fn init_share_memory(worker_num: u32) {}

#[napi]
pub fn worker_init(worker_num: u32, index: u32) {
    thread::spawn(move || {
        unsafe {
            WORKER_INDEX = index;
            MAX_WORKER_NUM = worker_num;
            let shm_size = MAX_WORKER_NUM * MAX_WORKER_NUM * MSG_CELL_SIZE;
            MAP_DESC = ipc::shm_init(shm_size);
            init_sema_map(worker_num, sema_oper::OPEN);
        }
    });
}

#[napi]
pub fn process_exit() {
    #[cfg(target_os = "windows")] unsafe {
        ipc::shm_clearup(MAP_DESC)
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
pub fn call_safe_func(callback: JsFunction) -> Result<()> {
    let tsfn: ThreadsafeFunction<u32, ErrorStrategy::CalleeHandled> = callback
        .create_threadsafe_function(0, |ctx| {
            let object = ctx.env.create_string("hello world!").unwrap();
            Ok(vec![object])
        })?;

    let one_second = time::Duration::from_secs(3);
    let tsfn = tsfn.clone();

    thread::spawn(move || {
        loop {
            unsafe {
                tsfn.call(Ok(0), ThreadsafeFunctionCallMode::NonBlocking);
                thread::sleep(one_second);
            }
        }
    });

    Ok(())
}

#[napi]
pub async fn call_node_func() -> Result<u32> {
    let one_second = time::Duration::from_secs(1);
    task::spawn_blocking(move || {
        thread::sleep(one_second);
    }).await.unwrap();
    return Ok(100);
}

fn clearup(env: Env) {
    println!("#shut down!!")
}

#[napi]
pub fn init(mut env: Env) -> Result<()> {
    env.add_env_cleanup_hook(env, clearup);
    //env.add_async_cleanup_hook(env, clearup);
    Ok(())
}



