use achordion_bank::factor::Factors;
use achordion_bank::waveform;
use achordion_lib::wavetable::Wavetable;

use super::FactorsRef;

static mut FACTORS_0: Option<Factors> = None;
static mut FACTORS_0_REF: Option<FactorsRef> = None;

static mut FACTORS_1: Option<Factors> = None;
static mut FACTORS_1_REF: Option<FactorsRef> = None;

static mut FACTORS_2: Option<Factors> = None;
static mut FACTORS_2_REF: Option<FactorsRef> = None;

static mut FACTORS_3: Option<Factors> = None;
static mut FACTORS_3_REF: Option<FactorsRef> = None;

static mut FACTORS_4: Option<Factors> = None;
static mut FACTORS_4_REF: Option<FactorsRef> = None;

static mut FACTORS_5: Option<Factors> = None;
static mut FACTORS_5_REF: Option<FactorsRef> = None;

pub static mut BANK: Option<[Wavetable<'static>; 6]> = None;

pub fn setup() {
    unsafe {
        FACTORS_0 = Some(Factors::from_raw(&waveform::harsh::HARSH_0));
        FACTORS_0_REF = Some(factors_ref!(FACTORS_0));

        FACTORS_1 = Some(Factors::from_raw(&waveform::harsh::HARSH_1));
        FACTORS_1_REF = Some(factors_ref!(FACTORS_1));

        FACTORS_2 = Some(Factors::from_raw(&waveform::harsh::HARSH_2));
        FACTORS_2_REF = Some(factors_ref!(FACTORS_2));

        FACTORS_3 = Some(Factors::from_raw(&waveform::harsh::HARSH_3));
        FACTORS_3_REF = Some(factors_ref!(FACTORS_3));

        FACTORS_4 = Some(Factors::from_raw(&waveform::harsh::HARSH_4));
        FACTORS_4_REF = Some(factors_ref!(FACTORS_4));

        FACTORS_5 = Some(Factors::from_raw(&waveform::harsh::HARSH_5));
        FACTORS_5_REF = Some(factors_ref!(FACTORS_5));

        let sample_rate = pd_sys::sys_getsr() as u32;
        BANK = Some([
            Wavetable::new(FACTORS_0_REF.as_ref().unwrap(), sample_rate),
            Wavetable::new(FACTORS_1_REF.as_ref().unwrap(), sample_rate),
            Wavetable::new(FACTORS_2_REF.as_ref().unwrap(), sample_rate),
            Wavetable::new(FACTORS_3_REF.as_ref().unwrap(), sample_rate),
            Wavetable::new(FACTORS_4_REF.as_ref().unwrap(), sample_rate),
            Wavetable::new(FACTORS_5_REF.as_ref().unwrap(), sample_rate),
        ]);
    }
}
