use std::os::raw::c_void;

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, tresult, FactoryFlags, IPluginFactory,
        IPluginFactory2, PClassInfo, PClassInfo2, PFactoryInfo,
    },
    VST3,
};

use crate::pi::parameters;
use crate::vst3::{
    controller::PiSynthController, plugin::PiSynthPlugin, plugin_data, utils::strcpy,
};

#[VST3(implements(IPluginFactory, IPluginFactory2))]
pub struct PiSynthPluginFactory {}

impl PiSynthPluginFactory {
    pub fn new() -> Box<Self> {
        Self::allocate()
    }
}

impl IPluginFactory for PiSynthPluginFactory {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        let info = &mut *info;

        strcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
        strcpy(plugin_data::VST3_URL, info.url.as_mut_ptr());
        strcpy(plugin_data::VST3_EMAIL, info.email.as_mut_ptr());

        info.flags = FactoryFlags::kComponentNonDiscardable as i32;

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        2
    }

    unsafe fn get_class_info(&self, idx: i32, info: *mut PClassInfo) -> tresult {
        match idx {
            0 => {
                let info = &mut *info;

                info.cardinality = 0x7FFF_FFFF;
                info.cid = PiSynthPlugin::CID;

                strcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());

                kResultOk
            }
            1 => {
                let info = &mut *info;

                info.cardinality = 0x7FFF_FFFF;
                info.cid = PiSynthController::CID;

                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_CATEGORY,
                    info.category.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );

                kResultOk
            }
            _ => kInvalidArgument,
        }
    }

    unsafe fn create_instance(
        &self,
        cid: *const GUID,
        _riid: *const GUID,
        obj: *mut *mut core::ffi::c_void,
    ) -> i32 {
        let iid = *cid;
        let param_info = parameters::make_parameter_info();

        match iid {
            PiSynthPlugin::CID => {
                let ptr = Box::into_raw(PiSynthPlugin::new(param_info)) as *mut c_void;
                *obj = ptr;

                kResultOk
            }
            PiSynthController::CID => {
                let ptr = Box::into_raw(PiSynthController::new(param_info)) as *mut c_void;
                *obj = ptr;

                kResultOk
            }
            _ => kResultFalse,
        }
    }
}

impl IPluginFactory2 for PiSynthPluginFactory {
    unsafe fn get_class_info2(&self, idx: i32, info: *mut PClassInfo2) -> tresult {
        match idx {
            0 => {
                let info = &mut *info;

                info.class_flags = 1;
                info.cardinality = 0x7FFF_FFFF;
                info.cid = PiSynthPlugin::CID;
                strcpy(plugin_data::VST3_CLASS_NAME, info.name.as_mut_ptr());
                strcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
                strcpy(plugin_data::VST3_VERSION, info.version.as_mut_ptr());
                strcpy(plugin_data::VST3_CLASS_CATEGORY, info.category.as_mut_ptr());
                strcpy(
                    plugin_data::VST3_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            1 => {
                let info = &mut *info;

                info.class_flags = 0;
                info.cardinality = 0x7FFF_FFFF;
                info.cid = PiSynthController::CID;
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_NAME,
                    info.name.as_mut_ptr(),
                );
                strcpy(plugin_data::VST3_VENDOR, info.vendor.as_mut_ptr());
                strcpy(plugin_data::VST3_VERSION, info.version.as_mut_ptr());
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_CATEGORY,
                    info.category.as_mut_ptr(),
                );
                strcpy(
                    plugin_data::VST3_CONTROLLER_CLASS_SUBCATEGORIES,
                    info.subcategories.as_mut_ptr(),
                );

                kResultOk
            }
            _ => return kInvalidArgument,
        }
    }
}
