use bindings::Windows::Win32::{
    Devices::{Communication::*, DeviceAndDriverInstallation::*},
    Foundation::{CloseHandle, HANDLE, HWND, PSTR},
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::*,
    System::{
        Diagnostics::Debug::GetLastError,
        SystemServices::*,
        Threading::{CreateEventA, WaitForSingleObject, WAIT_OBJECT_0},
    },
};
use std::{
    ffi::{c_void, CStr},
    ptr::null,
};
use windows::{Handle, IntoParam, Param};

pub struct ComPort(HANDLE);

impl Drop for ComPort {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
        self.0 = HANDLE(0);
    }
}

impl super::SerialPort for ComPort {
    fn list() -> Vec<String> {
        let mut ports = Vec::<String>::new();
        let set = unsafe {
            SetupDiGetClassDevsA(
                &GUID_DEVINTERFACE_COMPORT,
                PSTR(null::<u8>() as *mut u8),
                HWND(0),
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
            while SetupDiEnumDeviceInfo(set, i, &mut data).as_bool() {
                let u_str_ptr = &mut str_array as *mut u8;
                let i_str_ptr = u_str_ptr as *mut i8;
                SetupDiGetDeviceRegistryPropertyA(
                    set,
                    &mut data,
                    SPDRP_FRIENDLYNAME,
                    null::<u32>() as *mut u32,
                    u_str_ptr,
                    str_array.len() as u32,
                    null::<u32>() as *mut u32,
                );
                ports.push(CStr::from_ptr(i_str_ptr).to_str().unwrap().to_string());
                i += 1;
            }
            SetupDiDestroyDeviceInfoList(set);
        };
        ports
    }

    fn open(path: &str, baud: u32, timeout: u32) -> Result<Self, String> {
        let handle = unsafe {
            let p: Param<'_, PSTR> = path.into_param();
            let handle = CreateFileA(
                p.abi(),
                FILE_ACCESS_FLAGS(GENERIC_READ | GENERIC_WRITE),
                FILE_SHARE_MODE(0),
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
        let mut overlapped = OVERLAPPED::default();
        let mut read = 0u32;
        unsafe {
            overlapped.hEvent = CreateEventA(
                null::<SECURITY_ATTRIBUTES>() as *const SECURITY_ATTRIBUTES,
                true,
                false,
                PSTR(null::<u8>() as *mut u8),
            );
            ReadFile(
                self.0,
                buffer.as_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut read,
                &mut overlapped,
            );
            match WaitForSingleObject(overlapped.hEvent, u32::MAX) {
                WAIT_OBJECT_0 => {
                    if !GetOverlappedResult(self.0, &overlapped, &mut read, false).as_bool() {
                        None
                    } else {
                        Some(read as usize)
                    }
                }
                _ => None,
            }
        }
    }

    fn write(&self, buffer: &[u8]) -> Option<usize> {
        let mut overlapped = OVERLAPPED::default();
        let mut written = 0u32;
        unsafe {
            overlapped.hEvent = CreateEventA(
                null::<SECURITY_ATTRIBUTES>() as *const SECURITY_ATTRIBUTES,
                true,
                false,
                PSTR(null::<u8>() as *mut u8),
            );
            WriteFile(
                self.0,
                buffer.as_ptr() as *const c_void,
                buffer.len() as u32,
                &mut written,
                &mut overlapped,
            );
            match WaitForSingleObject(overlapped.hEvent, u32::MAX) {
                WAIT_OBJECT_0 => {
                    if !GetOverlappedResult(self.0, &overlapped, &mut written, false).as_bool() {
                        None
                    } else {
                        Some(written as usize)
                    }
                }
                _ => None,
            }
        }
    }
}
