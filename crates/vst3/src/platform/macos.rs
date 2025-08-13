use anyhow::Result;
use core_foundation::{
    base::{Boolean, CFRelease, TCFType},
    bundle::{
        CFBundleCreate, CFBundleGetFunctionPointerForName, CFBundleLoadExecutable, CFBundleRef,
    },
    string::CFString,
    url::{CFURLCreateWithFileSystemPath, kCFURLPOSIXPathStyle},
};

use crate::{VSTPtr, base::funknown::IPluginFactory};

pub struct Module {
    bundle: CFBundleRef,
}

impl Module {
    pub fn new(path: &str) -> Result<Self> {
        unsafe {
            let cf_path = CFString::new(path);
            let url = CFURLCreateWithFileSystemPath(
                std::ptr::null(),
                cf_path.as_concrete_TypeRef(),
                kCFURLPOSIXPathStyle,
                true as u8,
            );
            let bundle = CFBundleCreate(std::ptr::null(), url);
            if bundle.is_null() || CFBundleLoadExecutable(bundle) == 0 {
                return Err(anyhow::anyhow!("Failed to load bundle at {}", path));
            }

            let init_ptr = CFBundleGetFunctionPointerForName(
                bundle,
                CFString::new("bundleEntry").as_concrete_TypeRef(),
            );

            if init_ptr.is_null() {
                return Err(anyhow::anyhow!("bundleEntry not found"));
            }

            let init: unsafe extern "C" fn() = std::mem::transmute(init_ptr);
            init();

            Ok(Self { bundle })
        }
    }

    pub fn get_factory(&mut self) -> Result<VSTPtr<IPluginFactory>> {
        unsafe {
            let factory_ptr = CFBundleGetFunctionPointerForName(
                self.bundle,
                CFString::new("GetPluginFactory").as_concrete_TypeRef(),
            );
            if factory_ptr.is_null() {
                return Err(anyhow::anyhow!("GetPluginFactory not found"));
            }

            let get_factory: unsafe extern "C" fn() -> *mut IPluginFactory =
                std::mem::transmute(factory_ptr);

            Ok(VSTPtr::new(get_factory()))
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            let exit_ptr = CFBundleGetFunctionPointerForName(
                self.bundle,
                CFString::new("bundleExit").as_concrete_TypeRef(),
            );
            if !exit_ptr.is_null() {
                let exit: unsafe extern "C" fn() = std::mem::transmute(exit_ptr);
                exit();
            }

            CFRelease(self.bundle as *const _);
        }
    }
}
