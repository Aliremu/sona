#![allow(unused_variables)]
#![allow(warnings)]
#![warn(unused_imports)]

use crate::base::funknown::{
    FUnknown_Impl, IAudioProcessor_Impl, IComponent_Impl, IEditController_Impl, IPlugView_Impl,
    IPluginBase_Impl, IPluginFactory, IPluginFactory_Impl, Interface,
};
use crate::vst::host_application::{IConnectionPoint_Impl, IMessage_Impl};
use anyhow::Result;
use libc::c_char;
use libloading::{Library, Symbol};
use log::warn;
use std::error::Error;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Weak;

pub mod base;
pub mod gui;
pub mod platform;
pub mod vst;

#[cfg(target_os = "macos")]
pub use platform::macos::Module;

#[cfg(target_os = "windows")]
pub use platform::windows::Module;

#[derive(Debug, Clone, Copy)]
pub struct VSTPtr<T: FUnknown_Impl> {
    data: *mut T,
    _marker: PhantomData<T>,
}

impl<T: FUnknown_Impl> VSTPtr<T> {
    pub fn new(ptr: *mut T) -> Self {
        Self {
            data: ptr,
            _marker: PhantomData,
        }
    }

    pub fn as_weak(&self) -> Weak<T> {
        unsafe { Weak::from_raw(self.data) }
    }
}

impl<T: FUnknown_Impl> Deref for VSTPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.data) }
    }
}

impl<T: FUnknown_Impl> DerefMut for VSTPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.data) }
    }
}

unsafe impl<T: FUnknown_Impl> Sync for VSTPtr<T> {}
unsafe impl<T: FUnknown_Impl> Send for VSTPtr<T> {}

pub fn uid_to_ascii(uid: [c_char; 16]) -> String {
    // Convert [u8; 16] to a hex string (32 characters long)
    let hex_string = uid
        .iter()
        .map(|byte| format!("{:02X}", byte)) // Format each byte as 2 hex digits
        .collect::<String>();

    let formatted_uid = format!(
        "{}{}{}{}{}{}{}{}{}",
        &hex_string[0..8],
        "-",
        &hex_string[8..12],
        "-",
        &hex_string[12..16],
        "-",
        &hex_string[16..20],
        "-",
        &hex_string[20..32]
    );

    formatted_uid
}
