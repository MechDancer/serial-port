fn main() {
    windows::build! {
        Windows::Win32::Foundation::CloseHandle,
        Windows::Win32::System::Diagnostics::Debug::GetLastError,
        Windows::Win32::Storage::FileSystem::{
            CreateFileA,
            WriteFile,
            ReadFile,
        },
        Windows::Win32::System::SystemServices::{
            GENERIC_READ,
            GENERIC_WRITE,
            GUID_DEVINTERFACE_COMPORT,
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
