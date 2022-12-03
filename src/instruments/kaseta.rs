// TODO: Implement setting of control
// TODO: Implement hold button

use core::mem::MaybeUninit;
use rand::prelude::*;
use std::os::raw::{c_int, c_void};
use std::sync::Mutex;

use kaseta_control::{DesiredOutput, InputSnapshot, Store};
use kaseta_dsp::processor::Processor;
use kaseta_dsp::random::Random;
use sirena::memory_manager::MemoryManager;

use crate::{cstr, log};

static mut CLASS: Option<*mut pd_sys::_class> = None;
lazy_static! {
    static ref MEMORY_MANAGER: Mutex<MemoryManager> = {
        static mut MEMORY: [MaybeUninit<u32>; 48000 * 4 * 60 * 3] =
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
    right_outlet: *mut pd_sys::_outlet,
    led_1_outlet: *mut pd_sys::_outlet,
    led_2_outlet: *mut pd_sys::_outlet,
    led_3_outlet: *mut pd_sys::_outlet,
    led_4_outlet: *mut pd_sys::_outlet,
    led_5_outlet: *mut pd_sys::_outlet,
    led_6_outlet: *mut pd_sys::_outlet,
    led_7_outlet: *mut pd_sys::_outlet,
    led_8_outlet: *mut pd_sys::_outlet,
    led_9_outlet: *mut pd_sys::_outlet,
    impulse_outlet: *mut pd_sys::_outlet,
    input: InputSnapshot,
    control_connected: [bool; 4],
    output: DesiredOutput,
    cache: Store,
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
        number_of_outlets = 12,
        callback = perform
    );

    register_bang_method(class, tick);
    register_float_method(class, "control_1_connected", set_control_1_connected);
    register_float_method(class, "control_2_connected", set_control_2_connected);
    register_float_method(class, "control_3_connected", set_control_3_connected);
    register_float_method(class, "control_4_connected", set_control_4_connected);
    register_float_method(class, "control_1", set_control_1);
    register_float_method(class, "control_2", set_control_2);
    register_float_method(class, "control_3", set_control_3);
    register_float_method(class, "control_4", set_control_4);
    register_float_method(class, "button", set_button);
    register_float_method(class, "pre_amp", set_pre_amp);
    register_float_method(class, "dry_wet", set_dry_wet);
    register_float_method(class, "drive", set_drive);
    register_float_method(class, "bias", set_bias);
    register_float_method(class, "wow_flutter", set_wow_flut);
    register_float_method(class, "speed", set_speed);
    register_float_method(class, "tone", set_tone);
    register_float_method(class, "head_1_position", set_head_1_position);
    register_float_method(class, "head_2_position", set_head_2_position);
    register_float_method(class, "head_3_position", set_head_3_position);
    register_float_method(class, "head_4_position", set_head_4_position);
    register_float_method(class, "head_1_feedback", set_head_1_feedback);
    register_float_method(class, "head_2_feedback", set_head_2_feedback);
    register_float_method(class, "head_3_feedback", set_head_3_feedback);
    register_float_method(class, "head_4_feedback", set_head_4_feedback);
    register_float_method(class, "head_1_volume", set_head_1_volume);
    register_float_method(class, "head_2_volume", set_head_2_volume);
    register_float_method(class, "head_3_volume", set_head_3_volume);
    register_float_method(class, "head_4_volume", set_head_4_volume);
    register_float_method(class, "head_1_pan", set_head_1_pan);
    register_float_method(class, "head_2_pan", set_head_2_pan);
    register_float_method(class, "head_3_pan", set_head_3_pan);
    register_float_method(class, "head_4_pan", set_head_4_pan);
    register_float_method(class, "switch_1", set_option_1);
    register_float_method(class, "switch_2", set_option_2);
    register_float_method(class, "switch_3", set_option_3);
    register_float_method(class, "switch_4", set_option_4);
    register_float_method(class, "switch_5", set_option_5);
    register_float_method(class, "switch_6", set_option_6);
    register_float_method(class, "switch_7", set_option_7);
    register_float_method(class, "switch_8", set_option_8);
    register_float_method(class, "switch_9", set_option_9);
    register_float_method(class, "switch_10", set_option_10);
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

    let cache = Store::new();
    let processor = {
        let sample_rate = pd_sys::sys_getsr();
        // TODO: Do I need to initialize processor with attributes?
        Processor::new(sample_rate, &mut *MEMORY_MANAGER.lock().unwrap())
    };

    (*class).input = InputSnapshot::default();
    (*class).control_connected = [false; 4];
    (*class).cache = cache;
    (*class).processor = processor;

    pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).right_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_1_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_2_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_3_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_4_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_5_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_6_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_7_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_8_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).led_9_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).impulse_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);

    class as *mut c_void
}

unsafe fn register_bang_method(
    class: *mut pd_sys::_class,
    method: unsafe extern "C" fn(*mut Class),
) {
    pd_sys::class_addbang(
        class,
        Some(std::mem::transmute::<unsafe extern "C" fn(*mut Class), _>(
            method,
        )),
    );
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

unsafe extern "C" fn tick(class: *mut Class) {
    (*class).output = (*class).cache.tick();
}

macro_rules! set_control_connected {
    ( $name:ident, $index:expr ) => {
        unsafe extern "C" fn $name(class: *mut Class, value: f32) {
            let connected = value > 0.5;
            (*class).control_connected[$index] = connected;
        }
    };
}

set_control_connected!(set_control_1_connected, 0);
set_control_connected!(set_control_2_connected, 1);
set_control_connected!(set_control_3_connected, 2);
set_control_connected!(set_control_4_connected, 3);

macro_rules! set_control {
    ( $name:ident, $index:expr ) => {
        unsafe extern "C" fn $name(class: *mut Class, value: f32) {
            (*class).input.control[$index] = if (*class).control_connected[$index] {
                Some(value)
            } else {
                None
            };
            update_processor(class);
        }
    };
}

set_control!(set_control_1, 0);
set_control!(set_control_2, 1);
set_control!(set_control_3, 2);
set_control!(set_control_4, 3);

unsafe extern "C" fn set_button(class: *mut Class, value: f32) {
    let enabled = value > 0.5;
    (*class).input.button = enabled;
    update_processor(class);
}

unsafe extern "C" fn set_pre_amp(class: *mut Class, value: f32) {
    (*class).input.pre_amp = value;
    update_processor(class);
}

unsafe extern "C" fn set_dry_wet(class: *mut Class, value: f32) {
    (*class).input.dry_wet = value;
    update_processor(class);
}

unsafe extern "C" fn set_drive(class: *mut Class, value: f32) {
    (*class).input.drive = value;
    update_processor(class);
}

unsafe extern "C" fn set_bias(class: *mut Class, value: f32) {
    (*class).input.bias = value;
    update_processor(class);
}

unsafe extern "C" fn set_wow_flut(class: *mut Class, value: f32) {
    (*class).input.wow_flut = value;
    update_processor(class);
}

unsafe extern "C" fn set_speed(class: *mut Class, value: f32) {
    (*class).input.speed = value;
    update_processor(class);
}

unsafe extern "C" fn set_tone(class: *mut Class, value: f32) {
    (*class).input.tone = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_1_position(class: *mut Class, value: f32) {
    (*class).input.head[0].position = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_2_position(class: *mut Class, value: f32) {
    (*class).input.head[1].position = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_3_position(class: *mut Class, value: f32) {
    (*class).input.head[2].position = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_4_position(class: *mut Class, value: f32) {
    (*class).input.head[3].position = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_1_volume(class: *mut Class, value: f32) {
    (*class).input.head[0].volume = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_2_volume(class: *mut Class, value: f32) {
    (*class).input.head[1].volume = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_3_volume(class: *mut Class, value: f32) {
    (*class).input.head[2].volume = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_4_volume(class: *mut Class, value: f32) {
    (*class).input.head[3].volume = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_1_feedback(class: *mut Class, value: f32) {
    (*class).input.head[0].feedback = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_2_feedback(class: *mut Class, value: f32) {
    (*class).input.head[1].feedback = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_3_feedback(class: *mut Class, value: f32) {
    (*class).input.head[2].feedback = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_4_feedback(class: *mut Class, value: f32) {
    (*class).input.head[3].feedback = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_1_pan(class: *mut Class, value: f32) {
    (*class).input.head[0].pan = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_2_pan(class: *mut Class, value: f32) {
    (*class).input.head[1].pan = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_3_pan(class: *mut Class, value: f32) {
    (*class).input.head[2].pan = value;
    update_processor(class);
}

unsafe extern "C" fn set_head_4_pan(class: *mut Class, value: f32) {
    (*class).input.head[3].pan = value;
    update_processor(class);
}

macro_rules! set_option {
    ( $name:ident, $index:expr ) => {
        unsafe extern "C" fn $name(class: *mut Class, enabled: f32) {
            let enabled = enabled > 0.5;
            (*class).input.switch[$index] = enabled;
            update_processor(class);
        }
    };
}

set_option!(set_option_1, 0);
set_option!(set_option_2, 1);
set_option!(set_option_3, 2);
set_option!(set_option_4, 3);
set_option!(set_option_5, 4);
set_option!(set_option_6, 5);
set_option!(set_option_7, 6);
set_option!(set_option_8, 7);
set_option!(set_option_9, 8);
set_option!(set_option_10, 9);

unsafe fn update_processor(class: *mut Class) {
    let attributes = (*class)
        .cache
        .apply_input_snapshot((*class).input)
        .dsp_attributes;
    (*class).processor.set_attributes(attributes.into());
}

fn bool_to_f32(x: bool) -> f32 {
    if x {
        1.0
    } else {
        0.0
    }
}

fn perform(
    class: &mut Class,
    number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    const BUFFER_LEN: usize = 32;
    assert!(number_of_frames % BUFFER_LEN == 0);

    let mut buffer = [(0.0, 0.0); BUFFER_LEN];

    for chunk_index in 0..number_of_frames / BUFFER_LEN {
        for (i, frame) in buffer.iter_mut().enumerate() {
            let index = chunk_index * BUFFER_LEN + i;
            *frame = (inlets[0][index], 0.0);
        }

        let reaction = class.processor.process(&mut buffer, &mut KasetaRandom);
        class.cache.apply_dsp_reaction(reaction.into());

        for (i, frame) in buffer.iter().enumerate() {
            let index = chunk_index * BUFFER_LEN + i;
            (outlets[0][index], outlets[1][index]) = *frame;
            outlets[2][index] = bool_to_f32(class.output.display[0]);
            outlets[3][index] = bool_to_f32(class.output.display[1]);
            outlets[4][index] = bool_to_f32(class.output.display[2]);
            outlets[5][index] = bool_to_f32(class.output.display[3]);
            outlets[6][index] = bool_to_f32(class.output.display[4]);
            outlets[7][index] = bool_to_f32(class.output.display[5]);
            outlets[8][index] = bool_to_f32(class.output.display[6]);
            outlets[9][index] = bool_to_f32(class.output.display[7]);
            outlets[10][index] = bool_to_f32(class.output.impulse_led);
            outlets[11][index] = bool_to_f32(class.output.impulse_trigger);
        }
    }
}
