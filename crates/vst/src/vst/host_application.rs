use crate::base::funknown::{
    DefaultImplementation, FUID, FUnknown, FUnknown_HostImpl, FUnknown_Impl, FUnknown_Vtbl,
    IAudioProcessor, IComponent, Interface, Marker, TResult,
};
use libc::c_char;
use log::{error, warn};
use vst3_macro::{implement, interface};
use std::collections::HashMap;
use std::ffi::{CStr, c_void};

pub type String128 = [u16; 128];

#[interface(0x58E595CC, 0xDB2D4969, 0x8B6AAF8C, 0x36A664E5)]
pub trait IHostApplication: FUnknown {
    fn get_name(&mut self, name: String128) -> TResult;
    fn create_instance(&mut self, cid: FUID, iid: FUID, obj: *mut *mut c_void) -> TResult;
}

#[interface(0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6)]
pub trait IComponentHandler: FUnknown {
    fn begin_edit(&mut self, id: u32) -> TResult;
    fn perform_edit(&mut self, id: u32, value: u32) -> TResult;
    fn end_edit(&mut self, id: u32) -> TResult;
    fn restart_component(&mut self, flags: i32) -> TResult;
}

#[interface(0xF040B4B3, 0xA36045EC, 0xABCDC045, 0xB4D5A2CC)]
pub trait IComponentHandler2: FUnknown {
    fn set_dirty(&mut self, state: bool) -> TResult;
    fn request_open_editor(&mut self, name: *const c_char) -> TResult;
    fn start_group_edit(&mut self) -> TResult;
    fn finish_group_edit(&mut self) -> TResult;
}

#[interface(0x70A4156F, 0x6E6E4026, 0x989148BF, 0xAA60D8D1)]
pub trait IConnectionPoint: FUnknown {
    fn connect(&mut self, other: *mut IConnectionPoint) -> TResult;
    fn disconnect(&mut self, other: *mut IConnectionPoint) -> TResult;
    fn notify(&mut self, message: *mut c_void) -> TResult;
}

#[interface(0x936F033B, 0xC6C047DB, 0xBB0882F8, 0x13C1E613)]
pub trait IMessage: FUnknown {
    fn get_message_id(&mut self) -> *const c_char;
    fn set_message_id(&mut self, id: *const c_char) -> ();
    fn get_attributes(&mut self) -> *mut IAttributeList;
}

#[interface(0x1E5F0AEB, 0xCC7F4533, 0xA2544011, 0x38AD5EE4)]
pub trait IAttributeList: FUnknown {
    // Set integer value
    fn set_int(&mut self, id: *const c_char, value: i64) -> TResult;

    // Get integer value
    fn get_int(&mut self, id: *const c_char, value: &mut i64) -> TResult;

    // Set float value
    fn set_float(&mut self, id: *const c_char, value: f64) -> TResult;

    // Get float value
    fn get_float(&mut self, id: *const c_char, value: &mut f64) -> TResult;

    // Set string value (UTF-16, must be null-terminated)
    fn set_string(&mut self, id: *const c_char, string: *const u16) -> TResult;

    // Get string value (UTF-16), size is in bytes
    fn get_string(&mut self, id: *const c_char, string: *mut u16, size_in_bytes: u32) -> TResult;

    // Set binary data
    fn set_binary(&mut self, id: *const c_char, data: *const c_void, size_in_bytes: u32)
    -> TResult;

    // Get binary data
    fn get_binary(
        &mut self,
        id: *const c_char,
        data: &mut *const c_void,
        size_in_bytes: &mut u32,
    ) -> TResult;
}
