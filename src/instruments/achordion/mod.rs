mod bank;

use std::os::raw::{c_int, c_void};

use achordion_lib::instrument::Instrument;

use bank::WAVETABLE_BANKS;

use crate::{cstr, log};

static mut CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Class<'a> {
    pd_obj: pd_sys::t_object,
    solo_outlet: *mut pd_sys::_outlet,
    chord_outlet: *mut pd_sys::_outlet,
    instrument: Instrument<'a>,
    signal_dummy: f32,
}

#[no_mangle]
pub unsafe extern "C" fn achordion_tilde_setup() {
    bank::setup();

    let class = create_class();

    CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = Class,
        dummy_offset = offset_of!(Class => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 3,
        callback = perform
    );

    register_float_method(class, "solo", set_solo);
    register_float_method(class, "float", set_chord_root);
    register_float_method(class, "chord_degrees", set_chord_degrees);
    register_float_method(class, "scale_mode", set_scale_mode);
    register_float_method(class, "scale_root", set_scale_root);
    register_float_method(class, "wavetable_bank", set_wavetable_bank);
    register_float_method(class, "wavetable", set_wavetable);
    register_float_method(class, "detune", set_detune);
    register_float_method(class, "style", set_style);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[achordion~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("achordion~").as_ptr()),
        Some(new),
        None,
        std::mem::size_of::<Class>(),
        pd_sys::CLASS_DEFAULT as i32,
        0,
    )
}

unsafe extern "C" fn new() -> *mut c_void {
    let class = pd_sys::pd_new(CLASS.unwrap()) as *mut Class;

    let sample_rate = pd_sys::sys_getsr() as u32;
    let instrument = Instrument::new(&WAVETABLE_BANKS.as_ref().unwrap()[..], sample_rate);

    (*class).instrument = instrument;

    pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).solo_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);
    (*class).chord_outlet = pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);

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

unsafe extern "C" fn set_solo(class: *mut Class, value: pd_sys::t_float) {
    if value < 0.1 {
        (*class).instrument.set_solo_voct(None);
    } else {
        (*class)
            .instrument
            .set_solo_voct(Some(value.clamp(0.0, 10.0)));
    }
}

unsafe extern "C" fn set_chord_root(class: *mut Class, value: pd_sys::t_float) {
    (*class)
        .instrument
        .set_chord_root_linear(Some(value.clamp(0.0, 10.0)));
}

unsafe extern "C" fn set_chord_degrees(class: *mut Class, value: pd_sys::t_float) {
    (*class).instrument.set_chord_degrees(value.clamp(0.0, 1.0));
}

unsafe extern "C" fn set_scale_mode(class: *mut Class, value: pd_sys::t_float) {
    (*class)
        .instrument
        .set_scale_mode(value.clamp(0.0, 1.0), false);
}

unsafe extern "C" fn set_scale_root(class: *mut Class, value: pd_sys::t_float) {
    (*class)
        .instrument
        .set_scale_root_voct(value.clamp(0.0, 20.0));
}

unsafe extern "C" fn set_wavetable_bank(class: *mut Class, value: pd_sys::t_float) {
    (*class)
        .instrument
        .set_wavetable_bank(value.clamp(0.0, 1.0));
}

unsafe extern "C" fn set_wavetable(class: *mut Class, value: pd_sys::t_float) {
    (*class).instrument.set_wavetable(value.clamp(0.0, 1.0));
}

unsafe extern "C" fn set_detune(class: *mut Class, value: pd_sys::t_float) {
    (*class).instrument.set_detune(value.clamp(0.0, 1.0));
}

unsafe extern "C" fn set_style(class: *mut Class, value: pd_sys::t_float) {
    (*class).instrument.set_style(value.clamp(0.0, 1.0));
}

fn perform(
    class: &mut Class,
    _number_of_frames: usize,
    _inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    const BUFFER_LEN: usize = 32;
    assert!(outlets[0].len() % BUFFER_LEN == 0);

    let mut buffer_solo = [0.0; BUFFER_LEN];
    let mut buffer_chord = [0.0; BUFFER_LEN];

    for chunk_index in 0..outlets[0].len() / BUFFER_LEN {
        class
            .instrument
            .populate(&mut buffer_solo[..], &mut buffer_chord[..]);

        let start = chunk_index * BUFFER_LEN;
        for i in 0..BUFFER_LEN {
            outlets[1][start + i] = buffer_solo[i];
            outlets[2][start + i] = buffer_chord[i];
            outlets[0][start + i] = (outlets[1][start + i] + outlets[2][start + i]) / 2.0;
        }
    }
}
