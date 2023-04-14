use std::ffi::CString;
use winapi::shared::minwindef::{DWORD, FALSE, TRUE};
use winapi::shared::winerror::{WAIT_TIMEOUT};
use winapi::um::winbase::WAIT_FAILED;
use winapi::um::synchapi::{CreateSemaphoreW, OpenSemaphoreW, ReleaseSemaphore, WaitForSingleObject};
use winapi::um::winbase::WAIT_OBJECT_0;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use widestring::U16CString;
use winapi::um::winnt::HANDLE;
use std::os::raw::c_long;
use winapi::um::errhandlingapi::GetLastError;
use futures::StreamExt as _;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt};
use parity_tokio_ipc::{Endpoint, SecurityAttributes};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::um::winnt::{LONG, SEMAPHORE_ALL_ACCESS};


pub fn sema_create(name: &str) -> HANDLE {
    // 命名信号量名
    let semaphore_name: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0).into_iter()).collect();

    // 创建命名信号量，初始计数为0，最大计数为1
    let semaphore_handle: HANDLE = unsafe {
        CreateSemaphoreW(
            null_mut(),
            0,
            1,
            semaphore_name.as_ptr(),
        )
    };
    if semaphore_handle == null_mut() {
        panic!("CreateSemaphoreW failed");
    }

    println!("Semaphore created");
    return semaphore_handle;
}

pub fn sema_open(name: &str) -> HANDLE {
    // 命名信号量名
    let semaphore_name: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0).into_iter()).collect();

    // 打开命名信号量
    let semaphore_handle: HANDLE = unsafe {
        OpenSemaphoreW(
            SEMAPHORE_ALL_ACCESS,
            false as i32,
            semaphore_name.as_ptr(),
        )
    };
    if semaphore_handle == null_mut() {
        panic!("OpenSemaphoreW failed");
    }
    println!("Semaphore opened");
    return semaphore_handle;
}

pub fn sema_require(semaphore_handle: HANDLE) {
    // 获取信号量所有权
    let wait_result: DWORD = unsafe {
        WaitForSingleObject(semaphore_handle, 500000000)
    };
    match wait_result {
        WAIT_OBJECT_0 => {
            println!("Semaphore ownership acquired");
            // do something
        }
        _ => {
            println!("WaitForSingleObject failed");
        }
    }

    // 关闭命名信号量句柄
    let close_result: i32 = unsafe {
        CloseHandle(semaphore_handle)
    };
    if close_result == 0 {
        panic!("CloseHandle failed");
    }
}

pub fn sema_release(semaphore_handle: HANDLE) {
    // 释放信号量所有权
    let release_result: i32 = unsafe {
        ReleaseSemaphore(
            semaphore_handle,
            1,
            null_mut(),
        )
    };
    if release_result == 0 {
        panic!("ReleaseSemaphore failed");
    }
    println!("Semaphore ownership");
}

pub fn sema_close(semaphore: HANDLE) -> bool {
    let result = unsafe {
        CloseHandle(semaphore)
    };

    if result == FALSE {
        return false;
    }

    return true;
}