use memflow_win32::win32;

use super::kernel::FFIVirtualMemoryRef;

pub type Win32Process = win32::Win32Process<FFIVirtualMemoryRef>;
