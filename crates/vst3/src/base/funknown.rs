#![feature(negative_impls)]

use anyhow::anyhow;
use libc::c_char;
use log::warn;
use vst3_macro::interface;
use std::error::Error;
use std::ffi::{CStr, CString, c_float, c_void};
use std::fmt::Formatter;
use std::ops::Not;

use crate::gui::plug_view::{IPlugFrame, ViewRect};
use crate::uid_to_ascii;
use crate::vst::audio_processor::speaker_arr::SpeakerArrangement;
use crate::vst::audio_processor::{
    BusDirection, BusInfo, IoMode, MediaType, ProcessData, ProcessSetup, RoutingInfo,
    SymbolicSampleSize,
};

pub type FUID = [c_char; 16];

pub trait Marker<T> {}
pub trait DefaultImplementation<T> {}

pub trait Interface {
    type VTable: 'static;
    fn vtable(&self) -> &'static Self::VTable;

    #[allow(non_upper_case_globals)]
    const method_names: &'static [*const ()] = &[];

    #[allow(non_upper_case_globals)]
    const iid: FUID;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub enum FactoryFlags {
    #[default]
    NoFlags = 0,

    ClassesDiscardable = 1 << 0,
    LicenseCheck = 1 << 1,
    ComponentNonDiscardable = 1 << 3,
    Unicode = 1 << 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PFactoryInfo {
    pub vendor: [c_char; 64],
    pub url: [c_char; 256],
    pub email: [c_char; 128],
    pub flags: FactoryFlags,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PClassInfo {
    pub cid: [c_char; 16],
    pub cardinality: i32,
    pub category: [c_char; 32],
    pub name: [c_char; 64],
}

impl PClassInfo {
    pub unsafe fn category(&self) -> String {
        CStr::from_ptr(self.category.as_ptr())
            .to_str()
            .unwrap()
            .to_string()
    }

    pub unsafe fn name(&self) -> String {
        CStr::from_ptr(self.name.as_ptr())
            .to_str()
            .unwrap()
            .to_string()
    }
}

impl Default for PClassInfo {
    fn default() -> Self {
        Self {
            cid: [0; 16],
            cardinality: 0,
            category: [0; 32],
            name: [0; 64],
        }
    }
}

impl Default for PFactoryInfo {
    fn default() -> Self {
        Self {
            vendor: [0; 64],
            url: [0; 256],
            email: [0; 128],
            flags: FactoryFlags::NoFlags,
        }
    }
}

impl std::fmt::Display for PFactoryInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "PFactoryInfo {{ vendor: {:?}, url: {:?}, email: {:?}, flags: {:?} }}",
                CStr::from_ptr(self.vendor.as_ptr()),
                CStr::from_ptr(self.url.as_ptr()),
                CStr::from_ptr(self.email.as_ptr()),
                self.flags
            )
        }
    }
}

impl std::fmt::Display for PClassInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            write!(
                f,
                "PClassInfo {{ cid: {:?}, cardinality: {:?}, category: {:?}, name: {:?} }}",
                self.cid,
                self.cardinality,
                CStr::from_ptr(self.category.as_ptr()),
                CStr::from_ptr(self.name.as_ptr()),
            )
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum TResult {
    NoInterface = 0x80004002,     // E_NOINTERFACE
    ResultOk = 0,                 // S_OK
    ResultFalse = 1,              // S_FALSE
    InvalidArgument = 0x80070057, // E_INVALIDARG
    NotImplemented = 0x80004001,  // E_NOTIMPL
    InternalError = 0x80004005,   // E_FAIL
    NotInitialized = 0x8000FFFF,  // E_UNEXPECTED
    OutOfMemory = 0x8007000E,     // E_OUTOFMEMORY
}

impl std::fmt::Display for TResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TResult {}

#[repr(C)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct FUnknown {
    pub vtable: &'static FUnknown_Vtbl,
}

#[allow(non_camel_case_types)]
pub trait FUnknown_Impl: Interface {
    #[inline]
    unsafe fn query_interface<T: Interface>(&mut self) -> Result<&'static mut T, TResult>
    where
        <Self as Interface>::VTable: 'static,
    {
        let mut tmp: *mut c_void = std::ptr::null_mut();
        let res = (std::mem::transmute::<&'static Self::VTable, &FUnknown_Vtbl>(self.vtable())
            .query_interface)(self as *mut _ as *mut FUnknown, T::iid, &mut tmp);

        if res != TResult::ResultOk || tmp.is_null() {
            Err(res)
        } else {
            Ok(&mut *(tmp as *mut T))
        }
    }

    #[inline]
    unsafe fn add_ref(&mut self) -> u32
    where
        <Self as Interface>::VTable: 'static,
    {
        (std::mem::transmute::<&'static Self::VTable, &FUnknown_Vtbl>(self.vtable()).add_ref)(
            self as *mut _ as *mut FUnknown,
        )
    }

    #[inline]
    unsafe fn release(&mut self) -> u32
    where
        <Self as Interface>::VTable: 'static,
    {
        (std::mem::transmute::<&'static Self::VTable, &FUnknown_Vtbl>(self.vtable()).release)(
            self as *mut _ as *mut FUnknown,
        )
    }
}

#[allow(non_camel_case_types)]
pub trait FUnknown_HostImpl {
    #[inline]
    unsafe fn query_interface(&mut self, iid: FUID, obj: *mut *mut c_void) -> TResult {
        warn!("query_interface: {:?}, {:?}", uid_to_ascii(iid), iid);
        TResult::ResultOk
    }

    #[inline]
    unsafe fn add_ref(&mut self) -> u32 {
        // warn!("add_ref");
        1
    }

    #[inline]
    unsafe fn release(&mut self) -> u32 {
        // warn!("release");
        1
    }
}

impl<T: Interface + DefaultImplementation<FUnknown>> FUnknown_Impl for T {}
impl Interface for FUnknown {
    type VTable = FUnknown_Vtbl;
    const iid: FUID = [
        0i8, 0i8, 0i8, 0i8, 0i8, 0i8, 0i8, 0i8, -64i8, 0i8, 0i8, 0i8, 0i8, 0i8, 0i8, 70i8,
    ];
    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }
}
#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct FUnknown_Vtbl {
    pub query_interface: unsafe extern "C" fn(
        this: *mut FUnknown,
        iid: [c_char; 16],
        obj: *mut *mut c_void,
    ) -> TResult,
    pub add_ref: unsafe extern "C" fn(this: *mut FUnknown) -> u32,
    pub release: unsafe extern "C" fn(this: *mut FUnknown) -> u32,
}

#[interface(0x7A4D811C, 0x52114A1F, 0xAED9D2EE, 0x0B43BF9F)]
pub trait IPluginFactory: FUnknown {
    fn get_factory_info(&mut self, info: *mut PFactoryInfo) -> TResult;
    fn count_classes(&mut self) -> i32;

    #[private]
    fn get_class_info(&mut self, index: i32, info: *mut PClassInfo) -> TResult;

    fn get_class_info(&mut self, index: i32) -> Result<PClassInfo, TResult> {
        let mut class_info = PClassInfo::default();
        let res =
            (std::mem::transmute::<&'static Self::VTable, &IPluginFactory_Vtbl>(self.vtable())
                .get_class_info)(
                self as *const _ as *mut IPluginFactory,
                index,
                &mut class_info,
            );

        if res != TResult::ResultOk {
            Err(res)
        } else {
            Ok(class_info)
        }
    }

    #[private]
    fn create_instance(
        &mut self,
        cid: [c_char; 16],
        iid: [c_char; 16],
        obj: *mut *mut c_void,
    ) -> TResult;

    fn create_instance<T: Interface>(
        &mut self,
        cid: [c_char; 16],
    ) -> Result<&'static mut T, TResult> {
        let mut tmp: *mut c_void = std::ptr::null_mut();
        let res =
            (std::mem::transmute::<&'static Self::VTable, &IPluginFactory_Vtbl>(self.vtable())
                .create_instance)(
                self as *const _ as *mut IPluginFactory,
                cid,
                T::iid,
                &mut tmp,
            );

        if res != TResult::ResultOk || tmp.is_null() {
            Err(res)
        } else {
            Ok(&mut *(tmp as *mut T))
        }
    }
}

#[interface(0x22888DDB, 0x156E45AE, 0x8358B348, 0x08190625)]
pub trait IPluginBase: FUnknown {
    fn initialize(&mut self, context: *mut FUnknown) -> TResult;
    fn terminate(&mut self) -> TResult;
}

// #[interface(0x367FAF01, 0xAFA94693, 0x8D4DA2A0, 0xED0882A3)]
#[interface(0x5BC32507, 0xD06049EA, 0xA6151B52, 0x2B755B29)]
pub trait IPlugView: FUnknown {
    fn is_platform_type_supported(&mut self, ty: *const c_char) -> TResult;
    fn attached(&mut self, parent: *mut c_void, ty: *const c_char) -> TResult;
    fn removed(&mut self) -> TResult;

    fn on_wheel(&mut self, distance: f32) -> TResult;

    fn on_key_down(&mut self, key: u16, key_code: i16, modifiers: i16) -> TResult;
    fn on_key_up(&mut self, key: u16, key_code: i16, modifiers: i16) -> TResult;

    fn get_size(&mut self, size: *mut ViewRect) -> TResult;
    fn on_size(&mut self, new_size: *mut ViewRect) -> TResult;
    fn on_focus(&mut self, state: bool) -> TResult;

    fn set_frame(&mut self, frame: *mut IPlugFrame) -> TResult;

    fn can_resize(&mut self) -> TResult;
    fn check_size_constraint(&mut self, rect: *mut ViewRect) -> TResult;
}

#[interface(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D)]
pub trait IAudioProcessor: FUnknown {
    fn set_bus_arrangements(
        &mut self,
        inputs: *mut SpeakerArrangement,
        num_inputs: i32,
        outputs: *mut SpeakerArrangement,
        num_outputs: i32,
    ) -> TResult;
    fn get_bus_arrangements(
        &mut self,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> TResult;

    fn can_process_sample_size(&mut self, symbolic_sample_size: SymbolicSampleSize) -> TResult;

    fn get_latency_samples(&mut self) -> u32;

    fn setup_processing(&mut self, setup: *mut ProcessSetup) -> TResult;

    fn set_processing(&mut self, state: bool) -> TResult;

    fn process(&self, data: *mut ProcessData) -> TResult;

    fn get_tail_samples(&mut self) -> u32;
}

#[interface(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802)]
pub trait IComponent: IPluginBase {
    #[private]
    fn get_controller_class_id(&mut self, class_id: *mut FUID) -> TResult;

    fn get_controller_class_id(&mut self) -> Result<FUID, TResult> {
        let mut cid = FUID::default();
        let res =
            (std::mem::transmute::<&'static Self::VTable, &IComponent_Vtbl>(self.vtable())
                .get_controller_class_id)(self as *const _ as *mut IComponent, &mut cid);

        if res != TResult::ResultOk {
            Err(res)
        } else {
            Ok(cid)
        }
    }

    fn set_io_mode(&mut self, mode: IoMode) -> TResult;

    fn get_bus_count(&mut self, media_type: MediaType, dir: BusDirection) -> i32;

    fn get_bus_info(
        &mut self,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> TResult;

    fn get_routing_info(
        &mut self,
        in_info: *mut RoutingInfo,
        out_info: *mut RoutingInfo,
    ) -> TResult;

    fn activate_bus(
        &mut self,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        state: bool,
    ) -> TResult;

    fn set_active(&mut self, state: bool) -> TResult;

    fn set_state(&mut self, state: *mut c_void) -> TResult;

    fn get_state(&mut self, state: *mut c_void) -> TResult;
}

// #[interface(0x7F4EFE59, 0xF3204967, 0xAC27A3AE, 0xAFB63038)] #IEditController2

#[interface(0xDCD7BBE3, 0x7742448D, 0xA874AACC, 0x979C759E)]
pub trait IEditController: IPluginBase {
    fn set_component_state(&mut self, state: *mut c_void) -> TResult;

    fn set_state(&mut self, state: *mut c_void) -> TResult;

    fn get_state(&mut self, state: *mut c_void) -> TResult;

    fn get_parameter_count(&mut self) -> i32;

    fn get_parameter_info(&mut self, param_index: i32, info: *mut c_void) -> TResult;

    fn get_param_string_by_value(
        &mut self,
        id: u32,
        value_normalized: u32,
        string: *mut c_void,
    ) -> TResult;

    fn get_param_value_by_string(
        &mut self,
        id: u32,
        string: *mut c_void,
        value_normalized: *mut c_void,
    ) -> TResult;

    fn normalized_param_to_plain(&mut self, id: u32, value_normalized: u32) -> u32;

    fn plain_param_to_normalized(&mut self, id: u32, plain_value: u32) -> u32;

    fn get_param_normalized(&mut self, id: u32) -> u32;

    fn set_param_normalized(&mut self, id: u32, value: u32) -> TResult;

    fn set_component_handler(&mut self, handler: *mut c_void) -> TResult;

    fn create_view(&mut self, name: *const c_char) -> *mut IPlugView;
}

pub mod ViewType {
    use std::ffi::{CStr, c_char};
    pub const Editor: *const c_char =
        unsafe { CStr::from_bytes_with_nul_unchecked(b"editor\0") }.as_ptr();
}

#[interface(0x65ED9690, 0x8AC44525, 0x8AADEF7A, 0x72EA703F)]
pub trait IPlugViewContentScaleSupport: IPlugView {
    fn set_content_scale_factor(&mut self, scale_factor: f32) -> TResult;
}
