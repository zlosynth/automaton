use core::mem::MaybeUninit;
use rand::prelude::*;
use std::os::raw::{c_int, c_void};
use std::sync::Mutex;

use kaseta_control::{self as control, Cache, ControlAction};
use kaseta_dsp::processor::Processor;
use kaseta_dsp::random::Random;
use sirena::memory_manager::MemoryManager;

use crate::{cstr, log};

static mut CLASS: Option<*mut pd_sys::_class> = None;
lazy_static! {
    static ref MEMORY_MANAGER: Mutex<MemoryManager> = {
        static mut MEMORY: [MaybeUninit<u32>; 48000 * 4 * 60] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });
        Mutex::new(memory_manager)
    };
}

struct KasetaRandom;

impl Random for KasetaRandom {
    fn normal(&mut self) -> f32 {
        let mut rng = rand::thread_rng();
        rng.gen()
    }
}

#[repr(C)]
struct Class {
    pd_obj: pd_sys::t_object,
    led_1_outlet: *mut pd_sys::_outlet,
    led_2_outlet: *mut pd_sys::_outlet,
    led_3_outlet: *mut pd_sys::_outlet,
    led_4_outlet: *mut pd_sys::_outlet,
    led_5_outlet: *mut pd_sys::_outlet,
    led_6_outlet: *mut pd_sys::_outlet,
    led_7_outlet: *mut pd_sys::_outlet,
    led_8_outlet: *mut pd_sys::_outlet,
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
        number_of_outlets = 9,
        callback = perform
    );

    register_float_method(class, "pre_amp_pot", set_pre_amp_pot);
    register_float_method(class, "dry_wet_pot", set_dry_wet_pot);
    register_float_method(class, "drive_pot", set_drive_pot);
    register_float_method(class, "drive_cv", set_drive_cv);
    register_float_method(class, "bias_pot", set_bias_pot);
    register_float_method(class, "bias_cv", set_bias_cv);
    register_float_method(class, "wow_frequency_pot", set_wow_frequency_pot);
    register_float_method(class, "wow_frequency_cv", set_wow_frequency_cv);
    register_float_method(class, "wow_depth_pot", set_wow_depth_pot);
    register_float_method(class, "wow_depth_cv", set_wow_depth_cv);
    register_float_method(class, "wow_amp_noise", set_wow_amplitude_noise_pot);
    register_float_method(class, "wow_amp_spring", set_wow_amplitude_spring_pot);
    register_float_method(class, "wow_amp_filter", set_wow_filter_pot);
    register_float_method(class, "wow_phs_noise", set_wow_phase_noise_pot);
    register_float_method(class, "wow_phs_spring", set_wow_phase_spring_pot);
    register_float_method(class, "wow_phs_drift", set_wow_phase_drift_pot);
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
    register_float_method(class, "delay_range", set_delay_range);
    register_float_method(class, "delay_rewind_forward", set_delay_rewind_forward);
    register_float_method(class, "delay_rewind_backward", set_delay_rewind_backward);
    register_float_method(class, "delay_quantization_6", set_delay_quantization_6);
    register_float_method(class, "delay_quantization_8", set_delay_quantization_8);
    register_float_method(
        class,
        "delay_head_1_feedback_amp",
        set_delay_head_1_feedback_amount,
    );
    register_float_method(
        class,
        "delay_head_2_feedback_amp",
        set_delay_head_2_feedback_amount,
    );
    register_float_method(
        class,
        "delay_head_3_feedback_amp",
        set_delay_head_3_feedback_amount,
    );
    register_float_method(
        class,
        "delay_head_4_feedback_amp",
        set_delay_head_4_feedback_amount,
    );
    register_float_method(class, "delay_head_1_volume", set_delay_head_1_volume);
    register_float_method(class, "delay_head_2_volume", set_delay_head_2_volume);
    register_float_method(class, "delay_head_3_volume", set_delay_head_3_volume);
    register_float_method(class, "delay_head_4_volume", set_delay_head_4_volume);
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
    (*class).led_1_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_2_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_3_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_4_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_5_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_6_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_7_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_8_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);

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

unsafe extern "C" fn set_dry_wet_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDryWetPot(value));
}

unsafe extern "C" fn set_drive_pot(class: *mut Class, drive: f32) {
    apply_control_action(class, ControlAction::SetDrivePot(drive));
}

unsafe extern "C" fn set_drive_cv(class: *mut Class, drive: f32) {
    apply_control_action(class, ControlAction::SetDriveCV(drive));
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

unsafe extern "C" fn set_wow_amplitude_noise_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetWowAmplitudeNoisePot(value));
}

unsafe extern "C" fn set_wow_amplitude_spring_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetWowAmplitudeSpringPot(value));
}

unsafe extern "C" fn set_wow_filter_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetWowFilterPot(value));
}

unsafe extern "C" fn set_wow_phase_noise_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetWowPhaseNoisePot(value));
}

unsafe extern "C" fn set_wow_phase_spring_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetWowPhaseSpringPot(value));
}

unsafe extern "C" fn set_wow_phase_drift_pot(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetWowPhaseDriftPot(value));
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

unsafe extern "C" fn set_delay_range(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayRangeSwitch(enabled));
}

unsafe extern "C" fn set_delay_rewind_forward(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayRewindForwardSwitch(enabled));
}

unsafe extern "C" fn set_delay_rewind_backward(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayRewindBackwardSwitch(enabled));
}

unsafe extern "C" fn set_delay_quantization_6(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayQuantizationSix(enabled));
}

unsafe extern "C" fn set_delay_quantization_8(class: *mut Class, enabled: f32) {
    let enabled = enabled > 0.5;
    apply_control_action(class, ControlAction::SetDelayQuantizationEight(enabled));
}

unsafe extern "C" fn set_delay_head_1_feedback_amount(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadFeedbackAmount(0, value));
}

unsafe extern "C" fn set_delay_head_2_feedback_amount(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadFeedbackAmount(1, value));
}

unsafe extern "C" fn set_delay_head_3_feedback_amount(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadFeedbackAmount(2, value));
}

unsafe extern "C" fn set_delay_head_4_feedback_amount(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadFeedbackAmount(3, value));
}

unsafe extern "C" fn set_delay_head_1_volume(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadVolume(0, value));
}

unsafe extern "C" fn set_delay_head_2_volume(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadVolume(1, value));
}

unsafe extern "C" fn set_delay_head_3_volume(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadVolume(2, value));
}

unsafe extern "C" fn set_delay_head_4_volume(class: *mut Class, value: f32) {
    apply_control_action(class, ControlAction::SetDelayHeadVolume(3, value));
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

        let reaction = class.processor.process(&mut buffer, &mut KasetaRandom);
        let reaction = control::reduce_dsp_reaction(reaction, &mut (*class).cache);

        for (i, frame) in buffer.iter().enumerate() {
            let index = chunk_index * BUFFER_LEN + i;
            outlets[0][index] = *frame;
            outlets[1][index] = if reaction.leds[0] { 1.0 } else { 0.0 };
            outlets[2][index] = if reaction.leds[1] { 1.0 } else { 0.0 };
            outlets[3][index] = if reaction.leds[2] { 1.0 } else { 0.0 };
            outlets[4][index] = if reaction.leds[3] { 1.0 } else { 0.0 };
            outlets[5][index] = if reaction.leds[4] { 1.0 } else { 0.0 };
            outlets[6][index] = if reaction.leds[5] { 1.0 } else { 0.0 };
            outlets[7][index] = if reaction.leds[6] { 1.0 } else { 0.0 };
            outlets[8][index] = if reaction.leds[7] { 1.0 } else { 0.0 };
        }
    }
}
