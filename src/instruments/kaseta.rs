use std::os::raw::{c_int, c_void};

use kaseta_dsp::processor::Processor;

use crate::{cstr, log};

static mut CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Class {
    pd_obj: pd_sys::t_object,
    solo_outlet: *mut pd_sys::_outlet,
    chord_outlet: *mut pd_sys::_outlet,
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
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[kaseta~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("kaseta~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn() -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<Class>(),
        pd_sys::CLASS_DEFAULT as i32,
        0,
    )
}

unsafe extern "C" fn new() -> *mut c_void {
    let class = pd_sys::pd_new(CLASS.unwrap()) as *mut Class;

    let sample_rate = pd_sys::sys_getsr();
    let processor = Processor::new(sample_rate);

    (*class).processor = processor;

    pd_sys::outlet_new(&mut (*class).pd_obj, &mut pd_sys::s_signal);

    class as *mut c_void
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
