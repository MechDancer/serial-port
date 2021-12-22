use crate::{PortKey, SerialId, SerialPort};
use encoding::{all::GBK, DecoderTrap, Encoding};
use std::{
    ffi::{c_void, CStr},
    ptr::null,
};
use windows::Win32::{
    Devices::{
        Communication::{SetCommState, SetCommTimeouts, COMMTIMEOUTS, DCB},
        DeviceAndDriverInstallation::{
            SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsA,
            SetupDiGetDeviceRegistryPropertyA, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT,
            SPDRP_FRIENDLYNAME, SP_DEVINFO_DATA,
        },
    },
    Foundation::{CloseHandle, GetLastError, ERROR_IO_PENDING, HANDLE, PSTR},
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{CreateFileA, ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, OPEN_EXISTING},
    System::{
        Ioctl::GUID_DEVINTERFACE_COMPORT,
        SystemServices::{GENERIC_READ, GENERIC_WRITE},
        Threading::{CreateEventA, WaitForSingleObject, WAIT_OBJECT_0},
        IO::{GetOverlappedResult, OVERLAPPED},
    },
};

// https://docs.microsoft.com/en-us/previous-versions/ff802693(v=msdn.10)?redirectedfrom=MSDN

macro_rules! block_overlapped {
    ($oper:ident => $handle:expr => $buffer:ident) => {
        unsafe {
            let mut len = 0u32;
            let mut overlapped = OVERLAPPED {
                hEvent: CreateEventA(
                    null::<SECURITY_ATTRIBUTES>() as *const SECURITY_ATTRIBUTES,
                    true,
                    false,
                    PSTR(null::<u8>() as *mut u8),
                ),
                ..Default::default()
            };
            if $oper(
                $handle,
                $buffer.as_ptr() as *mut c_void,
                $buffer.len() as u32,
                &mut len,
                &mut overlapped,
            )
            .as_bool()
                || (GetLastError() == ERROR_IO_PENDING
                    && WaitForSingleObject(overlapped.hEvent, u32::MAX) == WAIT_OBJECT_0
                    && GetOverlappedResult($handle, &overlapped, &mut len, false).as_bool())
            {
                Some(len as usize)
            } else {
                None
            }
        }
    };
}

pub struct ComPort(HANDLE);

impl Drop for ComPort {
    fn drop(&mut self) {
        unsafe { CloseHandle(std::mem::replace(&mut self.0, HANDLE(0))) };
    }
}

impl SerialPort for ComPort {
    fn list() -> Vec<SerialId> {
        let mut ports = Vec::new();
        let set = unsafe {
            SetupDiGetClassDevsA(
                &GUID_DEVINTERFACE_COMPORT,
                PSTR(null::<u8>() as *mut u8),
                0,
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            )
            // if *set == INVALID_HANDLE_VALUE {}
        };

        let mut str_array = [0u8; 64];
        let mut i = 0;
        let mut data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        unsafe {
            // 列出名字
            while SetupDiEnumDeviceInfo(set, i, &mut data).as_bool() {
                let u_str_ptr = &mut str_array as *mut _;
                let i_str_ptr = u_str_ptr as *const _;
                SetupDiGetDeviceRegistryPropertyA(
                    set,
                    &mut data,
                    SPDRP_FRIENDLYNAME,
                    null::<u32>() as *mut u32,
                    u_str_ptr,
                    str_array.len() as u32,
                    null::<u32>() as *mut u32,
                );
                // 解析名字
                let name =
                    match GBK.decode(CStr::from_ptr(i_str_ptr).to_bytes(), DecoderTrap::Strict) {
                        Ok(s) => s,
                        Err(_) => CStr::from_ptr(i_str_ptr).to_string_lossy().to_string(),
                    };
                let (comment, num) = name
                    .rmatch_indices("COM")
                    .next()
                    .map(|m| (&name[..m.0 - 2], &name[m.0 + 3..name.len() - 1]))
                    .unwrap();
                if let Ok(n) = num.parse() {
                    ports.push(SerialId {
                        key: n,
                        comment: comment.into(),
                    });
                }
                i += 1;
            }
            SetupDiDestroyDeviceInfoList(set);
        };
        ports
    }

    fn open(path: &PortKey, baud: u32, timeout: u32) -> Result<Self, String> {
        let handle = unsafe {
            let mut path = format!("\\\\.\\COM{}\0", path);
            let handle = CreateFileA(
                PSTR(path.as_mut_ptr()),
                GENERIC_READ | GENERIC_WRITE,
                0,
                null::<SECURITY_ATTRIBUTES>() as *mut SECURITY_ATTRIBUTES,
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                HANDLE(0),
            );
            if handle.is_invalid() {
                return Err(format!("failed to open: {:?}", GetLastError()));
            }
            handle
        };

        let port = ComPort(handle);

        let mut dcb = DCB {
            DCBlength: std::mem::size_of::<DCB>() as u32,
            BaudRate: baud,
            ByteSize: 8,
            ..Default::default()
        };
        unsafe {
            if !SetCommState(port.0, &mut dcb).as_bool() {
                return Err(format!("failed to set dcb: {:?}", GetLastError()));
            }
        }

        let mut commtimeouts = COMMTIMEOUTS {
            ReadIntervalTimeout: 5,
            ReadTotalTimeoutConstant: timeout,
            ..Default::default()
        };
        unsafe {
            if !SetCommTimeouts(port.0, &mut commtimeouts).as_bool() {
                return Err(format!("failed to set timeout: {:?}", GetLastError()));
            }
        }

        Ok(port)
    }

    fn read(&self, buffer: &mut [u8]) -> Option<usize> {
        block_overlapped!(ReadFile => self.0 => buffer)
    }

    fn write(&self, buffer: &[u8]) -> Option<usize> {
        block_overlapped!(WriteFile => self.0 => buffer)
    }
}

#[test]
fn test_list() {
    println!("{:?}", ComPort::list());
}
