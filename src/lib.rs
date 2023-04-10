use futures::prelude::*;
use napi::bindgen_prelude::*;
use std::process;
use tokio::fs;
//use napi::tokio::{self, fs};
//#![deny(clippy::all)]

use futures::StreamExt as _;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

use parity_tokio_ipc::{Endpoint, SecurityAttributes};

use shared_memory::*;
use shared_memory::ShmemConf;
use std::sync::mpsc::channel;
use std::thread;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Option;
use std::path::Path;

use std::io::Error;
use std::os::raw::c_void;
use std::ptr::{null_mut};
use winapi::ctypes::c_ulong;

use winapi::um::memoryapi::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, OpenFileMappingW, FILE_MAP_ALL_ACCESS};
use winapi::um::winnt::{HANDLE, PAGE_READWRITE, SECTION_ALL_ACCESS, GENERIC_READ, GENERIC_WRITE, FILE_ALL_ACCESS};

use std::ptr;
use napi::{CallContext, JsNull, JsNumber};
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};

#[macro_use]
extern crate napi_derive;

// https://tokio.rs/tokio/tutorial/channels
// https://napi.rs/docs/concepts/async-task
// https://napi.rs/docs/concepts/async-fn
// https://napi.rs/docs/compat-mode/concepts/tokio

// https://docs.rs/interprocess/latest/interprocess/os/windows/named_pipe/enum.PipeDirection.html


#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
    a + b
}

#[napi]
pub fn show() -> u32 {
    process::id()
}

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
    let shmem_flink = Path::new("basic_mapping");
    let mut proc_table = ProcTable { proc_num: n, proc_list: vec![] };
    for i in 0..n - 1 {
        let proc_info = ProcInfo { process_id: 100, occupied: true };
        proc_table.proc_list.push(proc_info)
    }

    let bytes = bincode::serialize(&proc_table).unwrap();
    println!("{:?}", bytes);

    let shmem = match ShmemConf::new().size(bytes.len()).flink(shmem_flink).create() {
        Ok(m) => m,
        Err(ShmemError::LinkExists) => ShmemConf::new().flink(shmem_flink).open().unwrap(),
        Err(e) => {
            eprintln!("Unable to create or open shmem flink  : {e}");
            return;
        }
    };

    // Get pointer to the shared memory
    let mut raw_ptr = shmem.as_ptr();

    // WARNING: This is prone to race conditions as no sync/locking is used
    unsafe {
        for i in 0..bytes.len() {
            *(raw_ptr.offset(i as isize)) = bytes[i];
        }
    }

    println!("share memory create done!");
}

#[napi]
pub fn init_proc_info(brothers: u32, index: u32) {
    let shmem_flink = Path::new("basic_mapping");

    let mut proc_table = ProcTable { proc_num: brothers, proc_list: vec![] };
    for i in 0..brothers - 1 {
        let proc_info = ProcInfo { process_id: 100, occupied: true };
        proc_table.proc_list.push(proc_info)
    }

    let mut bytes = bincode::serialize(&proc_table).unwrap();
    println!("0: {:?}", bytes);

    let shmem = match ShmemConf::new().size(bytes.len()).flink(shmem_flink).create() {
        Ok(m) => {
            println!("create!");
            m
        }
        Err(ShmemError::LinkExists) => {
            println!("existed!");
            ShmemConf::new().flink(shmem_flink).open().unwrap()
        }
        Err(e) => {
            eprintln!("Unable to create or open shmem flink  : {e}");
            return;
        }
    };

    // Get pointer to the shared memory
    let mut raw_ptr = shmem.as_ptr();

    unsafe {
        for i in 0..bytes.len() {
            bytes[i] = *(raw_ptr.offset(i as isize));
        }
    }
    println!("1: {:?}", bytes);

    let proc_table: ProcTable = bincode::deserialize(&*bytes).unwrap();
    println!("## process number: {}", proc_table.proc_num);
    for i in 0..proc_table.proc_num {
        let proc_info = proc_table.proc_list.get(i as usize).unwrap();
        println!("## process id: {}, occupied: {}", proc_info.process_id, proc_info.process_id);
    }
}


fn do_shm_write() {
    let mapping_name = "RustMapping";
    let mapping_size = 1024;

    let handle = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            ptr::null_mut(),
            PAGE_READWRITE,
            0,
            mapping_size,
            mapping_name.encode_utf16().collect::<Vec<_>>().as_ptr(),
        )
    };

    if handle.is_null() {
        panic!("CreateFileMappingW failed");
    }

    let buffer = b"Hello, Rust";
    let data_ptr = buffer.as_ptr() as LPVOID;

    let map = unsafe {
        MapViewOfFile(
            handle,
            FILE_MAP_ALL_ACCESS,
            0,
            0,
            mapping_size as usize,
        )
    };

    if map.is_null() {
        panic!("MapViewOfFile failed");
    }

    unsafe {
        ptr::copy_nonoverlapping(data_ptr, map as *mut c_void, buffer.len());
        //UnmapViewOfFile(map);
        //CloseHandle(handle);
    }
}

#[napi]
pub fn test_shm_write_thread(callback: JsFunction) -> Result<()> {
    let one_second = time::Duration::from_secs(1);
    do_shm_write();

    thread::spawn(move || {
        loop {

            thread::sleep(one_second);
        }
    });
    Ok(())
}

#[napi]
fn test_shm_read_thread() {
    let mapping_name = "RustMapping";
    let mapping_size = 1024;

    let handle = unsafe {
        OpenFileMappingW(
            FILE_MAP_ALL_ACCESS,
            false.into(),
            mapping_name.encode_utf16().collect::<Vec<_>>().as_ptr(),
        )
    };

    unsafe {
        println!("last error: {}", GetLastError());
    }

    if handle.is_null() {
        panic!("OpenFileMappingW failed");
    }

    let map = unsafe {
        MapViewOfFile(
            handle,
            FILE_MAP_ALL_ACCESS,
            0,
            0,
            mapping_size as usize,
        )
    };

    if map.is_null() {
        panic!("MapViewOfFile failed");
    }

    let buffer = unsafe {
        let slice = std::slice::from_raw_parts(map as *const u8, mapping_size as usize);
        std::str::from_utf8_unchecked(slice)
    };

    println!("Read from shared memory: {}", buffer);

    unsafe {
        UnmapViewOfFile(map);
        CloseHandle(handle);
    }
}



use napi::{JsFunction, Result};
use std::{time};
use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

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