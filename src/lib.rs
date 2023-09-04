#![deny(clippy::all)]

use std::ffi::CString;

use napi::bindgen_prelude::Buffer;
use std::mem::zeroed;
use std::ptr::null_mut;
use winapi::shared::minwindef::{DWORD, FALSE, TRUE};
use winapi::shared::ntdef::NULL;
use winapi::shared::winerror::{ERROR_IO_INCOMPLETE, ERROR_IO_PENDING};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{CreateFileA, ReadFile, WriteFile, OPEN_EXISTING};
use winapi::um::handleapi::CloseHandle;
use winapi::um::ioapiset::GetOverlappedResult;
use winapi::um::minwinbase::OVERLAPPED;
use winapi::um::winbase::{FILE_FLAG_NO_BUFFERING, FILE_FLAG_OVERLAPPED};
use winapi::um::winnt::{
  FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE,
};

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn send_usb(path: String, buffer: Buffer) -> String {
  let path = CString::new(path).unwrap();
  let access = GENERIC_READ | GENERIC_WRITE;
  let share_mode = FILE_SHARE_READ | FILE_SHARE_WRITE;
  let creation_disposition = OPEN_EXISTING;
  let flags_and_attributes = FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED | FILE_FLAG_NO_BUFFERING;
  let handle = unsafe {
    CreateFileA(
      path.as_ptr(),
      access,
      share_mode,
      null_mut(),
      creation_disposition,
      flags_and_attributes,
      NULL,
    )
  };

  // 发送并接收数据
  let mut overlapped = unsafe { zeroed::<OVERLAPPED>() };
  let mut bytes_written: DWORD = 0;
  let mut bytes_read: DWORD = 0;

  let mut res_buffer: Vec<u8> = vec![0; 1024];
  let mut ret = unsafe {
    WriteFile(
      handle,
      buffer.as_ptr() as *const _,
      buffer.len() as u32,
      &mut bytes_written,
      &mut overlapped,
    )
  };
  if ret == FALSE {
    let err = unsafe { GetLastError() };
    if err != ERROR_IO_PENDING && err != ERROR_IO_INCOMPLETE {
      return format!("err: {:?}", err);
    }
  }
  ret = unsafe { GetOverlappedResult(handle, &mut overlapped, &mut bytes_written, TRUE) };
  if ret == FALSE {
    return format!("err: {:?}", unsafe { GetLastError() });
  }

  ret = unsafe {
    ReadFile(
      handle,
      res_buffer.as_mut_ptr() as *mut _,
      res_buffer.len() as u32,
      &mut bytes_read,
      &mut overlapped,
    )
  };
  if ret == FALSE {
    let err = unsafe { GetLastError() };
    if err != ERROR_IO_PENDING && err != ERROR_IO_INCOMPLETE {
      return format!("err: {:?}", err);
    }
  }

  ret = unsafe { GetOverlappedResult(handle, &mut overlapped, &mut bytes_read, TRUE) };

  if ret == FALSE {
    return format!("err: {:?}", unsafe { GetLastError() });
  }
  unsafe {
    CloseHandle(handle);
  }

  // 返回接收到的数据
  println!("bytes_read: {:?}", bytes_read);
  res_buffer.truncate(bytes_read as usize);

  // res_buffer转换成u8类型

  let res_str = String::from_utf8_lossy(&res_buffer).to_string();

  res_str
}
