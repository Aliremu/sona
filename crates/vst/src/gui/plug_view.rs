use vst3_macro::interface;

use crate::base::funknown::{
    DefaultImplementation, FUID, FUnknown, FUnknown_Impl, FUnknown_Vtbl, IPlugView, Interface,
    Marker, TResult,
};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ViewRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub mod PlatformType {
    use std::ffi::{CStr, c_char};

    pub const HWND: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"HWND\0") }.as_ptr();

    pub const HIView: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"HIView\0") }.as_ptr();

    pub const NSView: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"NSView\0") }.as_ptr();

    pub const UIView: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"UIView\0") }.as_ptr();
}

#[interface(0x367FAF01, 0xAFA94693, 0x8D4DA2A0, 0xED0882A3)]
pub trait IPlugFrame: FUnknown {
    fn resize_view(&mut self, view: *mut IPlugView, new_size: *mut ViewRect) -> TResult;
}
