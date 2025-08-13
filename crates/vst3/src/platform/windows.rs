use anyhow::Result;
use libloading::{Library, Symbol};
use crate::{VSTPtr, base::funknown::IPluginFactory};

type InitDllProc = fn() -> bool;
type ExitDllProc = fn() -> bool;
type GetPluginFactoryProc = fn() -> *mut IPluginFactory;

pub struct Module {
    lib: Option<Library>,
}

impl Module {
    pub fn new(path: &str) -> Result<Self> {
        unsafe {
            let lib = Library::new(path)?;
            let init: Symbol<InitDllProc> = lib.get(b"InitDll")?;
            init();

            Ok(Self { lib: Some(lib) })
        }
    }

    pub fn get_factory(&mut self) -> Result<VSTPtr<IPluginFactory>> {
        unsafe {
            let raw_factory: Symbol<GetPluginFactoryProc> = self
                .lib
                .as_ref()
                .expect("Library is None!")
                .get::<GetPluginFactoryProc>(b"GetPluginFactory")?;

            Ok(VSTPtr::new(raw_factory()))
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            let mut lib = self.lib.take().unwrap();
            let exit: Symbol<ExitDllProc> = lib.get(b"ExitDll").unwrap();
            exit();

            lib.close().unwrap();
        }
    }
}
