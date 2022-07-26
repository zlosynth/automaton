#![deny(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate field_offset;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod wrapper;

pub mod instruments;

mod cstr;
mod log;

use std::os::raw::c_void;

use instruments::achordion;
use instruments::kaseta;

static mut AUTOMATON_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Automaton {
    _pd_obj: pd_sys::t_object,
}

unsafe extern "C" fn automaton_new() -> *mut c_void {
    let counter = pd_sys::pd_new(AUTOMATON_CLASS.unwrap()) as *mut Automaton;

    counter as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn automaton_setup() {
    log::info("[automaton] initializing");

    let class = create_class();

    AUTOMATON_CLASS = Some(class);

    achordion::achordion_tilde_setup();
    kaseta::kaseta_tilde_setup();
}

unsafe fn create_class() -> *mut pd_sys::_class {
    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("automaton").as_ptr()),
        Some(automaton_new),
        None,
        std::mem::size_of::<Automaton>(),
        pd_sys::CLASS_NOINLET as i32,
        0,
    )
}
