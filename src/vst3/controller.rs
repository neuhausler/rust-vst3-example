use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem;

use std::os::raw::c_void;
use std::ptr::null_mut;

use vst3_com::sys::GUID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultFalse, kResultOk, kResultTrue, tresult, FIDString, IBStream,
        IPluginBase,
    },
    utils::VstPtr,
    vst::{
        kRootUnitId, IEditController, IUnitInfo, ParameterFlags, ParameterInfo, ProgramListInfo,
        TChar, UnitInfo,
    },
    ComPtr, VST3,
};

use crate::pi::parameters::{Normalizable, Parameter, PiParameter};
use crate::vst3::{plugin_data, utils};

#[VST3(implements(IEditController, IUnitInfo))]
pub struct PiSynthController {
    pi_params: HashMap<Parameter, PiParameter>,
    vst3_params: RefCell<HashMap<u32, ParameterInfo>>,
    param_values: RefCell<HashMap<u32, f64>>,
}

impl PiSynthController {
    pub const CID: GUID = GUID {
        data: plugin_data::VST3_CONTROLLER_CID,
    };

    unsafe fn add_parameter(
        &self,
        id: u32,
        title: &str,
        short_title: &str,
        units: &str,
        step_count: i32,
        default_value: f64,
        flags: i32,
    ) {
        let mut vst3_params = self.vst3_params.borrow_mut();
        let mut param_vals = self.param_values.borrow_mut();

        let mut param = utils::make_empty_param_info();
        param.id = id;
        utils::wstrcpy(title, param.title.as_mut_ptr());
        utils::wstrcpy(short_title, param.short_title.as_mut_ptr());
        utils::wstrcpy(units, param.units.as_mut_ptr());
        param.step_count = step_count;
        param.default_normalized_value = default_value;
        param.unit_id = kRootUnitId;
        param.flags = flags;

        (*vst3_params).insert(id, param);
        (*param_vals).insert(id, param.default_normalized_value);
    }

    pub unsafe fn new(pi_params: HashMap<Parameter, PiParameter>) -> Box<PiSynthController> {
        let vst3_params = RefCell::new(HashMap::new());
        let param_vals = RefCell::new(HashMap::new());

        PiSynthController::allocate(pi_params, vst3_params, param_vals)
    }
}

impl IPluginBase for PiSynthController {
    unsafe fn initialize(&self, _host_context: *mut c_void) -> tresult {
        let pi_params = self.pi_params.clone();
        for (param, pi_param) in pi_params.iter() {
            self.add_parameter(
                *param as u32,
                &pi_param.title,
                &pi_param.short_title,
                &pi_param.unit_name,
                pi_param.step_count,
                pi_param.default_value,
                ParameterFlags::kCanAutomate as i32,
            );
        }

        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl IEditController for PiSynthController {
    unsafe fn set_component_state(&self, state: *mut c_void) -> tresult {
        if state.is_null() {
            return kResultFalse;
        }

        let state = state as *mut *mut _;
        let state: ComPtr<dyn IBStream> = ComPtr::new(state);

        let mut num_bytes_read = 0;
        for param in Parameter::iter() {
            let mut value = 0.0;
            let ptr = &mut value as *mut f64 as *mut c_void;

            state.read(ptr, mem::size_of::<f64>() as i32, &mut num_bytes_read);
            self.param_values.borrow_mut().insert(param as u32, value);
        }

        kResultOk
    }

    unsafe fn set_state(&self, _state: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn get_state(&self, _state: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn get_parameter_count(&self) -> i32 {
        self.vst3_params.borrow().len() as i32
    }

    unsafe fn get_parameter_info(&self, id: i32, vst3_params: *mut ParameterInfo) -> tresult {
        let id = id as u32;

        if let Some(param) = self.vst3_params.borrow().get(&id) {
            *vst3_params = *param;

            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn get_param_string_by_value(
        &self,
        id: u32,
        value_normalized: f64,
        string: *mut TChar,
    ) -> tresult {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.pi_params.get(&param) {
                    utils::tcharcpy(&p.format(value_normalized), string)
                } else {
                    return kResultFalse;
                }
            }
            _ => (),
        }

        kResultOk
    }

    unsafe fn get_param_value_by_string(
        &self,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> tresult {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.pi_params.get(&param) {
                    if let Some(v) = p.parse(&utils::tchar_to_string(string)) {
                        *value_normalized = v;
                    } else {
                        return kResultFalse;
                    }
                } else {
                    return kResultFalse;
                }
            }
            _ => (),
        }
        kResultOk
    }

    unsafe fn normalized_param_to_plain(&self, id: u32, value_normalized: f64) -> f64 {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.pi_params.get(&param) {
                    p.denormalize(value_normalized)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    unsafe fn plain_param_to_normalized(&self, id: u32, value_plain: f64) -> f64 {
        match Parameter::try_from(id) {
            Ok(param) => {
                if let Some(p) = self.pi_params.get(&param) {
                    p.normalize(value_plain)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    unsafe fn get_param_normalized(&self, id: u32) -> f64 {
        match self.param_values.borrow().get(&id) {
            Some(val) => *val,
            _ => 0.0,
        }
    }

    unsafe fn set_param_normalized(&self, id: u32, value: f64) -> tresult {
        match self.param_values.borrow_mut().insert(id, value) {
            Some(_) => kResultTrue,
            _ => kResultFalse,
        }
    }

    unsafe fn set_component_handler(&self, _handler: *mut c_void) -> tresult {
        kResultOk
    }

    unsafe fn create_view(&self, _name: FIDString) -> *mut c_void {
        null_mut()
    }
}

impl IUnitInfo for PiSynthController {
    unsafe fn get_unit_count(&self) -> i32 {
        1
    }

    unsafe fn get_unit_info(&self, _unit_index: i32, _info: *mut UnitInfo) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_list_count(&self) -> i32 {
        0
    }

    unsafe fn get_program_list_info(&self, _list_index: i32, _info: *mut ProgramListInfo) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_name(&self, _list_id: i32, _program_index: i32, _name: *mut u16) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_info(
        &self,
        _list_id: i32,
        _program_index: i32,
        _attribute_id: *const u8,
        _attribute_value: *mut u16,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn has_program_pitch_names(&self, _id: i32, _index: i32) -> i32 {
        kResultFalse
    }

    unsafe fn get_program_pitch_name(
        &self,
        _id: i32,
        _index: i32,
        _pitch: i16,
        _name: *mut u16,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn get_selected_unit(&self) -> i32 {
        0
    }

    unsafe fn select_unit(&self, _id: i32) -> i32 {
        kResultFalse
    }

    unsafe fn get_unit_by_bus(
        &self,
        _type_: i32,
        _dir: i32,
        _index: i32,
        _channel: i32,
        _unit_id: *mut i32,
    ) -> i32 {
        kResultFalse
    }

    unsafe fn set_unit_program_data(
        &self,
        _list_or_unit: i32,
        _program_index: i32,
        _data: VstPtr<dyn IBStream>,
    ) -> i32 {
        kResultFalse
    }
}
