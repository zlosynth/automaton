use core::mem::MaybeUninit;
use std::os::raw::{c_int, c_void};
use std::sync::Mutex;

use kaseta_control::{self as control, Cache, ControlAction};
use kaseta_dsp::processor::Processor;
use sirena::memory_manager::MemoryManager;

use crate::{cstr, log};

static mut CLASS: Option<*mut pd_sys::_class> = None;
lazy_static! {
    static ref MEMORY_MANAGER: Mutex<MemoryManager> = {
        static mut MEMORY: [MaybeUninit<u32>; 48000 * 4 * 20] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });
        Mutex::new(memory_manager)
    };
}

#[repr(C)]
struct Class {
    pd_obj: pd_sys::t_object,
    solo_outlet: *mut pd_sys::_outlet,
    chord_outlet: *mut pd_sys::_outlet,
    cache: Cache,
    processor: Processor,
    signal_dummy: f32,
}

#[no_mangle]
pub unsafe extern "C" fn kaseta_tilde_setup() {
    let class = create_class();

    CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = Class,
        dummy_offset = offset_of!(Class => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 1,
        callback = perform
    );

    register_float_method(class, "pre_amp_pot", set_pre_amp_pot);
    register_float_method(class, "drive_pot", set_drive_pot);
    register_float_method(class, "drive_cv", set_drive_cv);
    register_float_method(class, "saturation_pot", set_saturation_pot);
    register_float_method(class, "saturation_cv", set_saturation_cv);
    register_float_method(class, "bias_pot", set_bias_pot);
    register_float_method(class, "bias_cv", set_bias_cv);
    register_float_method(class, "wow_frequency_pot", set_wow_frequency_pot);
    register_float_method(class, "wow_frequency_cv", set_wow_frequency_cv);
    register_float_method(class, "wow_depth_pot", set_wow_depth_pot);
    register_float_method(class, "wow_depth_cv", set_wow_depth_cv);
    register_float_method(class, "delay_length_pot", set_delay_length_pot);
    register_float_method(class, "delay_length_cv", set_delay_length_cv);
    register_float_method(
        class,
        "delay_head_1_position_pot",
        set_delay_head_1_position_pot,
    );
    register_float_method(
        class,
        "delay_head_1_position_cv",
        set_delay_head_1_position_cv,
    );
    register_float_method(
        class,
        "delay_head_2_position_pot",
        set_delay_head_2_position_pot,
    );
    register_float_method(
        class,
        "delay_head_2_position_cv",
        set_delay_head_2_position_cv,
    );
    register_float_method(
        class,
        "delay_head_3_position_pot",
        set_delay_head_3_position_pot,
    );
    register_float_method(
        class,
        "delay_head_3_position_cv",
        set_delay_head_3_position_cv,
    );
    register_float_method(
        class,
        "delay_head_4_position_pot",
        set_delay_head_4_position_pot,
    );
    register_float_method(
        class,
        "delay_head_4_position_cv",
        set_delay_head_4_position_cv,
    );
    register_float_method(class, "delay_quantization_6", set_delay_quantization_6);
    register_float_method(class, "delay_quantization_8", set_delay_quantization_8);
    register_float_method(class, "delay_head_1_play", set_delay_head_1_play);
    register_float_method(class, "delay_head_2_play", set_delay_head_2_play);
    register_float_method(class, "delay_head_3_play", set_delay_head_3_play);
    register_float_method(class, "delay_head_4_play", set_delay_head_4_play);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[kaseta~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("kaseta~").as_ptr()),
        Some(new),
        None,
        std::mem::size_of::<Class>(),
        pd_sys::CLASS_DEFAULT as i32,
        0,
    )
}

unsafe extern "C" fn new() -> *mut c_void {
    let class = pd_sys::pd_new(CLASS.unwrap()) as *mut Class;

    let cache = Cache::default();
    let processor = {
        let sample_rate = pd_sys::sys_getsr();
        let mut processor = Processor::new(sample_rate, &mut *MEMORY_MANAGER.lock().unwrap());
        let attributes = control::cook_dsp_reaction_from_cache(&cache).into();
        processor.set_attributes(attributes);
        processor
    };

    (*class).cache = cache;
    (*class).processor = processor;

    pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);

    class as *mut c_void
}

unsafe fn register_float_method(
    class: *mut pd_sys::_class,
    symbol: &str,
    method: unsafe extern "C" fn(*mut Class, pd_sys::t_float),
) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Class, pd_sys::t_float),
            _,
        >(method)),
        pd_sys::gensym(cstr::cstr(symbol).as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe extern "C" fn set_pre_amp_pot(class: *mut Class, pre_amp: f32) {
    apply_control_action(class, ControlAction::SetPreAmpPot(pre_amp));
}

unsafe extern "C" fn set_drive_pot(class: *mut Class, drive: f32) {
    apply_control_action(class, ControlAction::SetDrivePot(drive));
}

unsafe extern "C" fn set_drive_cv(class: *mut Class, drive: f32) {
    apply_control_action(class, ControlAction::SetDriveCV(drive));
}

unsafe extern "C" fn set_saturation_pot(class: *mut Class, saturation: f32) {
    apply_control_action(class, ControlAction::SetSaturationPot(saturation));
}

unsafe extern "C" fn set_saturation_cv(class: *mut Class, saturation: f32) {
    apply_control_action(class, ControlAction::SetSaturationCV(saturation));
}

unsafe extern "C" fn set_bias_pot(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetBiasPot(bias));
}

unsafe extern "C" fn set_bias_cv(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetBiasCV(bias));
}

unsafe extern "C" fn set_wow_frequency_pot(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetWowFrequencyPot(bias));
}

unsafe extern "C" fn set_wow_frequency_cv(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetWowFrequencyCV(bias));
}

unsafe extern "C" fn set_wow_depth_pot(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetWowDepthPot(bias));
}

unsafe extern "C" fn set_wow_depth_cv(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetWowDepthCV(bias));
}

unsafe extern "C" fn set_delay_length_cv(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetDelayLengthCV(bias));
}

unsafe extern "C" fn set_delay_length_pot(class: *mut Class, bias: f32) {
    apply_control_action(class, ControlAction::SetDelayLengthPot(bias));
}

unsafe extern "C" fn set_delay_head_1_position_cv(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionCV(0, position));
}

unsafe extern "C" fn set_delay_head_1_position_pot(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionPot(0, position));
}

unsafe extern "C" fn set_delay_head_2_position_cv(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionCV(1, position));
}

unsafe extern "C" fn set_delay_head_2_position_pot(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionPot(1, position));
}

unsafe extern "C" fn set_delay_head_3_position_cv(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionCV(2, position));
}

unsafe extern "C" fn set_delay_head_3_position_pot(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionPot(2, position));
}

unsafe extern "C" fn set_delay_head_4_position_cv(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionCV(3, position));
}

unsafe extern "C" fn set_delay_head_4_position_pot(class: *mut Class, position: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadPositionPot(3, position));
}

unsafe extern "C" fn set_delay_quantization_6(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayQuantizationSix(enabled));
}

unsafe extern "C" fn set_delay_quantization_8(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayQuantizationEight(enabled));
}

unsafe extern "C" fn set_delay_head_1_play(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayHeadPlay(0, enabled));
}

unsafe extern "C" fn set_delay_head_2_play(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayHeadPlay(1, enabled));
}

unsafe extern "C" fn set_delay_head_3_play(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayHeadPlay(2, enabled));
}

unsafe extern "C" fn set_delay_head_4_play(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayHeadPlay(3, enabled));
}

unsafe fn apply_control_action(class: *mut Class, action: ControlAction) {
    let dsp_reaction = control::reduce_control_action(action, &mut (*class).cache);
    (*class).processor.set_attributes(dsp_reaction.into());
}

fn perform(
    class: &mut Class,
    number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    const BUFFER_LEN: usize = 32;
    assert!(number_of_frames % BUFFER_LEN == 0);

    let mut buffer = [0.0; BUFFER_LEN];

    for chunk_index in 0..number_of_frames / BUFFER_LEN {
        for (i, frame) in buffer.iter_mut().enumerate() {
            let index = chunk_index * BUFFER_LEN + i;
            *frame = inlets[0][index];
        }

        class.processor.process(&mut buffer);

        for (i, frame) in buffer.iter().enumerate() {
            let index = chunk_index * BUFFER_LEN + i;
            outlets[0][index] = *frame;
        }
    }
}
