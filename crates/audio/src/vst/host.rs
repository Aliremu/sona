#![allow(unused_variables)]

use std::{
    ffi::{c_char, c_void, CStr},
    sync::Arc,
};

use anyhow::Result;
use log::{info, trace, warn};
use vst3::{
    base::funknown::{
        FUnknown, FUnknown_HostImpl, FUnknown_Impl, IAudioProcessor, IAudioProcessor_Impl,
        IComponent, IComponent_Impl, IEditController, IEditController_Impl, IPlugView,
        IPlugView_Impl, IPluginBase_Impl, IPluginFactory, IPluginFactory_Impl, Interface,
        PFactoryInfo, TResult, ViewType, FUID,
    },
    gui::plug_view::{IPlugFrame, IPlugFrame_HostImpl, ViewRect},
    uid_to_ascii,
    vst::{
        audio_processor::{
            BusDirection, BusInfo, IParameterChanges_HostImpl, IoMode, MediaType, ProcessMode,
            ProcessSetup, SymbolicSampleSize,
        },
        host_application::{
            IAttributeList, IAttributeList_HostImpl, IComponentHandler2,
            IComponentHandler2_HostImpl, IComponentHandler_HostImpl, IConnectionPoint,
            IConnectionPoint_Impl, IHostApplication, IHostApplication_HostImpl, IMessage,
            IMessage_HostImpl, String128,
        },
    },
    Module, VSTPtr,
};
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct VSTHostContext {
    pub name: String,
    pub module: Option<Module>,
    pub factory: Option<VSTPtr<IPluginFactory>>,
    pub component: Option<VSTPtr<IComponent>>,
    pub processor: Option<VSTPtr<IAudioProcessor>>,
    pub editor: Option<VSTPtr<IEditController>>,
    pub view: Option<VSTPtr<IPlugView>>,

    pub host: Option<Arc<VSTHostApplication>>,
    pub handler: Option<Arc<HostComponentHandler>>,

    pub component_connection: Option<VSTPtr<IConnectionPoint>>,
    pub controller_connection: Option<VSTPtr<IConnectionPoint>>,
}

unsafe impl Sync for VSTHostContext {}
unsafe impl Send for VSTHostContext {}

impl VSTHostContext {
    pub fn new(path: &str) -> Result<Self> {
        unsafe {
            let mut module = Module::new(path)?;
            let factory = module.get_factory()?;

            let mut ctx = Self::default();

            let mut factory_info = PFactoryInfo::default();
            factory.get_factory_info(&mut factory_info);

            info!("Loaded plugin! {}", factory_info);

            let host = Arc::new(VSTHostApplication::new());
            let handler = Arc::new(HostComponentHandler::new());

            let context = Arc::into_raw(host.clone()) as *mut FUnknown;

            ctx.host = Some(host.clone());
            ctx.handler = Some(handler.clone());

            for i in 0..factory.count_classes() {
                let class_info = factory.get_class_info(i)?;

                if class_info.category() != "Audio Module Class" {
                    continue;
                }

                ctx.name = class_info.name();

                let comp = factory.create_instance::<IComponent>(class_info.cid)?;
                comp.set_io_mode(IoMode::Advanced);

                let edit = match comp.get_controller_class_id() {
                    Ok(edit_cid) => {
                        trace!("Initializing create_instance!");
                        factory.create_instance::<IEditController>(edit_cid)?
                    }

                    Err(err) => {
                        trace!("Initializing query_interface! {:?}", err);
                        comp.query_interface::<IEditController>()?
                    }
                };

                let _ = comp.initialize(context);

                let component_connection = comp.query_interface::<IConnectionPoint>()?;
                let controller_connection = edit.query_interface::<IConnectionPoint>()?;

                trace!("Component Connection: {:?}", component_connection);
                trace!("Controller Connection: {:?}", controller_connection);

                component_connection.connect(controller_connection);
                controller_connection.connect(component_connection);

                trace!("Setting up processor!");

                let processor = comp.query_interface::<IAudioProcessor>()?;

                trace!("Processor: {:?}", processor);

                let mut data = ProcessSetup {
                    process_mode: ProcessMode::Realtime,
                    symbolic_sample_size: SymbolicSampleSize::Sample32,
                    max_samples_per_block: 2048,
                    sample_rate: 44100.0,
                };
                let res = processor.setup_processing(&mut data);

                trace!(
                    "AudioProcessor: {:?}. Setup Processing: {:?}",
                    processor, res
                );

                for i in 0..comp.get_bus_count(MediaType::Audio, BusDirection::Input) {
                    let mut bus_info = BusInfo::default();
                    comp.get_bus_info(MediaType::Audio, BusDirection::Input, i, &mut bus_info);

                    trace!("Bus Input Info: {:?}", bus_info);
                }

                for i in 0..comp.get_bus_count(MediaType::Audio, BusDirection::Output) {
                    let mut bus_info = BusInfo::default();
                    comp.get_bus_info(MediaType::Audio, BusDirection::Output, i, &mut bus_info);

                    trace!("Bus Output Info: {:?}", bus_info);
                }

                comp.activate_bus(MediaType::Audio, BusDirection::Input, 0, true);
                comp.activate_bus(MediaType::Audio, BusDirection::Output, 0, true);

                comp.set_active(true);

                trace!("Parameter count: {}", edit.get_parameter_count());

                trace!("Initializing editor controller!");
                let res = edit.initialize(context);

                trace!("Setting command handler!");
                let res = edit.set_component_handler(Arc::into_raw(handler.clone()) as *mut _);

                let view = edit.create_view(ViewType::Editor);
                (*(view)).set_frame(
                    Box::into_raw(Box::new(HostPlugFrame::new())) as *mut _ as *mut IPlugFrame
                );

                warn!(
                    "{} {:?} {:?} {:?} {:?}",
                    class_info, comp as *mut _, edit as *mut _, context as *mut _, view
                );

                ctx.controller_connection = Some(VSTPtr::new(controller_connection));
                ctx.component_connection = Some(VSTPtr::new(component_connection));
                ctx.component = Some(VSTPtr::new(comp));
                ctx.processor = Some(VSTPtr::new(processor));
                ctx.editor = Some(VSTPtr::new(edit));
                ctx.view = Some(VSTPtr::new(view));
            }

            ctx.factory = Some(factory);
            ctx.module = Some(module);

            Ok(ctx)
        }
    }
}

impl Drop for VSTHostContext {
    fn drop(&mut self) {
        unsafe {
            warn!("Dropping VSTHostContext!");

            self.controller_connection.take().unwrap().release();
            self.component_connection.take().unwrap().release();
            self.view.unwrap().removed();
            self.view.take().unwrap().release();
            self.editor.take().unwrap().release();
            self.processor.take().unwrap().release();
            self.component.take().unwrap().release();
            self.factory.take().unwrap().release();

            drop(self.module.take());
        }
    }
}

#[repr(C)]
pub struct VSTHostApplication {
    vtable: &'static [*const (); 5],
}

impl VSTHostApplication {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const _,
                <Self as FUnknown_HostImpl>::add_ref as *const _,
                <Self as FUnknown_HostImpl>::release as *const _,
                <Self as IHostApplication_HostImpl>::get_name as *const _,
                <Self as IHostApplication_HostImpl>::create_instance as *const _,
            ],
        }
    }
}

impl Interface for VSTHostApplication {
    type VTable = [*const (); 5];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [1; 16];
}

impl FUnknown_HostImpl for VSTHostApplication {
    unsafe fn query_interface(&mut self, iid: FUID, obj: *mut *mut c_void) -> TResult {
        warn!(
            "VSTHostApplication:query_interface: {:?}, {:?}",
            uid_to_ascii(iid),
            iid
        );
        if iid == IHostApplication::iid {
            *obj = self as *mut _ as *mut c_void;
        } else {
            let host = Box::new(HostComponentHandler::new());
            *obj = Box::into_raw(host) as *mut c_void;
        }

        TResult::ResultOk
    }
}

impl IHostApplication_HostImpl for VSTHostApplication {
    unsafe fn get_name(&mut self, mut name: String128) -> TResult {
        warn!("get_name");
        name[0] = 'V' as u16;
        name[1] = 'o' as u16;
        name[2] = 'x' as u16;
        name[3] = 'e' as u16;
        name[4] = 'a' as u16;
        name[1] = '\0' as u16;
        TResult::ResultOk
    }

    unsafe fn create_instance(&mut self, cid: FUID, iid: FUID, obj: *mut *mut c_void) -> TResult {
        // warn!("create_instance: {:?}", uid_to_ascii(iid));

        if iid == IMessage::iid {
            // warn!("Creating message!");
            let message = Box::new(HostMessage::new());
            *obj = Box::into_raw(message) as *mut c_void;
        }

        TResult::ResultOk
    }
}

#[repr(C)]
pub struct HostComponentHandler {
    vtable: &'static [*const (); 11],
}

impl HostComponentHandler {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const _,
                <Self as FUnknown_HostImpl>::add_ref as *const _,
                <Self as FUnknown_HostImpl>::release as *const _,
                <Self as IComponentHandler_HostImpl>::begin_edit as *const _,
                <Self as IComponentHandler_HostImpl>::perform_edit as *const _,
                <Self as IComponentHandler_HostImpl>::end_edit as *const _,
                <Self as IComponentHandler_HostImpl>::restart_component as *const _,
                <Self as IComponentHandler2_HostImpl>::set_dirty as *const _,
                <Self as IComponentHandler2_HostImpl>::request_open_editor as *const _,
                <Self as IComponentHandler2_HostImpl>::start_group_edit as *const _,
                <Self as IComponentHandler2_HostImpl>::finish_group_edit as *const _,
            ],
        }
    }
}

impl Interface for HostComponentHandler {
    type VTable = [*const (); 11];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [2; 16];
}

impl FUnknown_HostImpl for HostComponentHandler {
    unsafe fn query_interface(&mut self, iid: FUID, obj: *mut *mut c_void) -> TResult {
        warn!(
            "HostComponentHandler:query_interface: {:?}",
            uid_to_ascii(iid)
        );

        if iid == IHostApplication::iid {
            let host = Box::new(VSTHostApplication::new());
            *obj = Box::into_raw(host) as *mut c_void;
        } else if iid == IComponentHandler2::iid {
            *obj = self as *mut _ as *mut c_void;
        }

        TResult::ResultOk
    }
}

impl IComponentHandler_HostImpl for HostComponentHandler {
    unsafe fn begin_edit(&mut self, id: u32) -> TResult {
        warn!("begin_edit: {:?}", id);
        TResult::ResultOk
    }

    unsafe fn perform_edit(&mut self, id: u32, value: u32) -> TResult {
        warn!("perform_edit: {:?}", id);
        TResult::ResultOk
    }

    unsafe fn end_edit(&mut self, id: u32) -> TResult {
        warn!("end_edit: {:?}", id);
        TResult::ResultOk
    }

    unsafe fn restart_component(&mut self, flags: i32) -> TResult {
        warn!("restart_component");
        TResult::NoInterface
    }
}

impl IComponentHandler2_HostImpl for HostComponentHandler {
    unsafe fn set_dirty(&mut self, state: bool) -> TResult {
        warn!("set_dirty");
        TResult::ResultOk
    }

    unsafe fn request_open_editor(&mut self, name: *const c_char) -> TResult {
        warn!("request_open_editor");
        TResult::ResultOk
    }

    unsafe fn start_group_edit(&mut self) -> TResult {
        warn!("start_group_edit");
        TResult::ResultOk
    }

    unsafe fn finish_group_edit(&mut self) -> TResult {
        warn!("finish_group_edit");
        TResult::ResultOk
    }
}

#[repr(C)]
pub struct HostPlugFrame {
    vtable: &'static [*const (); 4],
}

impl HostPlugFrame {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const _,
                <Self as FUnknown_HostImpl>::add_ref as *const _,
                <Self as FUnknown_HostImpl>::release as *const _,
                <Self as IPlugFrame_HostImpl>::resize_view as *const _,
            ],
        }
    }
}

impl Interface for HostPlugFrame {
    type VTable = [*const (); 4];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [3; 16];
}

impl FUnknown_HostImpl for HostPlugFrame {}

impl IPlugFrame_HostImpl for HostPlugFrame {
    unsafe fn resize_view(&mut self, view: *mut IPlugView, new_size: *mut ViewRect) -> TResult {
        warn!("HostPlugFrame::resize_view: {:?}", *new_size);
        TResult::ResultOk
    }
}

#[repr(C)]
pub struct HostMessage {
    vtable: &'static [*const (); 6],
    message_id: *const c_char,
    attributes: HostApplicationList,
}

impl HostMessage {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const (),
                <Self as FUnknown_HostImpl>::add_ref as *const (),
                <Self as FUnknown_HostImpl>::release as *const (),
                <Self as IMessage_HostImpl>::get_message_id as *const (),
                <Self as IMessage_HostImpl>::set_message_id as *const (),
                <Self as IMessage_HostImpl>::get_attributes as *const (),
            ],
            message_id: std::ptr::null(),
            attributes: HostApplicationList::new(),
        }
    }
}

impl Interface for HostMessage {
    type VTable = [*const (); 6];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [4; 16];
}

impl FUnknown_HostImpl for HostMessage {}

impl IMessage_HostImpl for HostMessage {
    unsafe fn get_message_id(&mut self) -> *const c_char {
        // warn!("get_message_id");
        self.message_id
    }

    unsafe fn set_message_id(&mut self, id: *const c_char) -> () {
        // warn!("set_message_id");
        self.message_id = id;
    }

    unsafe fn get_attributes(&mut self) -> *mut IAttributeList {
        // warn!("get_attributes");
        &mut (self.attributes) as *mut _ as *mut IAttributeList
    }
}

#[repr(C)]
#[derive(Debug)]
pub enum ParamValue {
    IntValue(i64),
    FloatValue(f64),
    CStrValue(*const c_char),
    BinaryValue(*const c_void),
}

#[repr(C)]
pub struct HostApplicationList {
    vtable: &'static [*const (); 11],
    list: FxHashMap<String, ParamValue>,
}

impl HostApplicationList {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const (),
                <Self as FUnknown_HostImpl>::add_ref as *const (),
                <Self as FUnknown_HostImpl>::release as *const (),
                <Self as IAttributeList_HostImpl>::set_int as *const (),
                <Self as IAttributeList_HostImpl>::get_int as *const (),
                <Self as IAttributeList_HostImpl>::set_float as *const (),
                <Self as IAttributeList_HostImpl>::get_float as *const (),
                <Self as IAttributeList_HostImpl>::set_string as *const (),
                <Self as IAttributeList_HostImpl>::get_string as *const (),
                <Self as IAttributeList_HostImpl>::set_binary as *const (),
                <Self as IAttributeList_HostImpl>::get_binary as *const (),
            ],

            list: FxHashMap::default(),
        }
    }
}

impl Interface for HostApplicationList {
    type VTable = [*const (); 11];

    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [5; 16];
}

impl FUnknown_HostImpl for HostApplicationList {}

impl IAttributeList_HostImpl for HostApplicationList {
    unsafe fn set_int(&mut self, id: *const c_char, value: i64) -> TResult {
        // warn!("set_int: {:?}", CStr::from_ptr(id).to_str().unwrap().to_string());

        self.list.insert(
            CStr::from_ptr(id).to_str().unwrap().to_string(),
            ParamValue::IntValue(value),
        );
        TResult::ResultOk
    }

    unsafe fn get_int(&mut self, id: *const c_char, value: &mut i64) -> TResult {
        let id = CStr::from_ptr(id).to_str().unwrap();
        // warn!("get_int: {:?}", id);
        // warn!("Map: {:#?}", self.list);
        if let Some(ParamValue::IntValue(val)) = self.list.get(id) {
            // warn!("\"{:?}\" -> {:?}", id, val);
            *value = *val;
        }
        TResult::ResultOk
    }

    unsafe fn set_float(&mut self, id: *const c_char, value: f64) -> TResult {
        // warn!("set_float");

        self.list.insert(
            CStr::from_ptr(id).to_str().unwrap().to_string(),
            ParamValue::FloatValue(value),
        );
        TResult::ResultOk
    }

    unsafe fn get_float(&mut self, id: *const c_char, value: &mut f64) -> TResult {
        // warn!("get_float");

        let id = CStr::from_ptr(id).to_str().unwrap();
        if let Some(ParamValue::FloatValue(val)) = self.list.get(id) {
            // warn!("\"{:?}\" -> {:?}", id, val);
            *value = *val;
        }
        TResult::ResultOk
    }

    unsafe fn set_string(&mut self, id: *const c_char, string: *const u16) -> TResult {
        warn!("set_string");
        TResult::ResultOk
    }

    unsafe fn get_string(
        &mut self,
        id: *const c_char,
        string: *mut u16,
        size_in_bytes: u32,
    ) -> TResult {
        warn!("get_string");
        TResult::ResultOk
    }

    unsafe fn set_binary(
        &mut self,
        id: *const c_char,
        data: *const c_void,
        size_in_bytes: u32,
    ) -> TResult {
        warn!("set_binary");
        TResult::ResultOk
    }

    unsafe fn get_binary(
        &mut self,
        id: *const c_char,
        data: &mut *const c_void,
        size_in_bytes: &mut u32,
    ) -> TResult {
        warn!("get_binary");
        TResult::ResultOk
    }
}

#[repr(C)]
pub struct HostParameterChanges {
    vtable: &'static [*const (); 6],
}

impl HostParameterChanges {
    pub fn new() -> Self {
        Self {
            vtable: &[
                <Self as FUnknown_HostImpl>::query_interface as *const (),
                <Self as FUnknown_HostImpl>::add_ref as *const (),
                <Self as FUnknown_HostImpl>::release as *const (),
                <Self as IParameterChanges_HostImpl>::get_parameter_count as *const (),
                <Self as IParameterChanges_HostImpl>::get_parameter_data as *const (),
                <Self as IParameterChanges_HostImpl>::add_parameter_data as *const (),
            ],
        }
    }
}

impl Interface for HostParameterChanges {
    type VTable = [*const (); 6];
    fn vtable(&self) -> &'static Self::VTable {
        self.vtable
    }

    const iid: FUID = [6; 16];
}

impl FUnknown_HostImpl for HostParameterChanges {}

impl IParameterChanges_HostImpl for HostParameterChanges {
    unsafe fn get_parameter_count(&mut self) -> i32 {
        // warn!("get_parameter_count");
        0
    }

    unsafe fn get_parameter_data(&mut self) -> *mut c_void {
        warn!("get_parameter_data");
        std::ptr::null_mut()
    }

    unsafe fn add_parameter_data(&mut self, id: *const c_char, index: *mut i32) -> *mut c_void {
        warn!("add_parameter_data");
        std::ptr::null_mut()
    }
}
