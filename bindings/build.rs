fn main() {
    // https://microsoft.github.io/windows-docs-rs/doc/windows/
    windows::runtime::build! {
        Windows::Win32::Foundation::{CloseHandle, GetLastError},
        Windows::Win32::Storage::FileSystem::{
            CreateFileA,
            WriteFile,
            ReadFile,
        },
        Windows::Win32::System::{
            Threading::{CreateEventA, WaitForSingleObject, WAIT_OBJECT_0},
            Ioctl::GUID_DEVINTERFACE_COMPORT,
            IO::{OVERLAPPED, GetOverlappedResult},
            SystemServices::{
                GENERIC_READ,
                GENERIC_WRITE,
            }
        },
        Windows::Win32::Devices::Communication::{
            SetCommState,
            SetCommTimeouts,
        },
        Windows::Win32::Devices::DeviceAndDriverInstallation::{
            SetupDiGetClassDevsA,
            SetupDiEnumDeviceInfo,
            SetupDiGetDeviceRegistryPropertyA,
            SetupDiDestroyDeviceInfoList,

            DIGCF_PRESENT,
            DIGCF_DEVICEINTERFACE,

            SPDRP_FRIENDLYNAME,
        },
    }
}
