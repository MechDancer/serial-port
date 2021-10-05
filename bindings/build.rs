fn main() {
    windows::build! {
        Windows::Win32::Foundation::CloseHandle,
        Windows::Win32::Storage::FileSystem::{
            CreateFileA,
            WriteFile,
            ReadFile,
        },
        Windows::Win32::System::{
            Diagnostics::Debug::GetLastError,
            Threading::{CreateEventA, WaitForSingleObject},
            SystemServices::{
                GENERIC_READ,
                GENERIC_WRITE,
                GUID_DEVINTERFACE_COMPORT,
                GetOverlappedResult,
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
