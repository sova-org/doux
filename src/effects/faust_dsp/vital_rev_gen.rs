/* ------------------------------------------------------------
name: "vital_rev"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn VitalRevDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct VitalRevDsp {
	fSampleRate: i32,
	fConst0: F32,
	fHslider0: F32,
	fHslider1: F32,
	iVec0: [i32;2],
	fConst1: F32,
	fHslider2: F32,
	fRec18: [F32;2],
	fHslider3: F32,
	fConst2: F32,
	fHslider4: F32,
	fHslider5: F32,
	fRec20: [F32;2],
	fHslider6: F32,
	fRec21: [F32;2],
	fVec1: [F32;2],
	fRec19: [F32;2],
	fHslider7: F32,
	fRec22: [F32;2],
	IOTA0: i32,
	fVec2: [F32;131072],
	fConst3: F32,
	fHslider8: F32,
	fHslider9: F32,
	fConst4: F32,
	fHslider10: F32,
	fVec3: [F32;65536],
	fVec4: [F32;2],
	fRec24: [F32;2],
	fRec23: [F32;2],
	fVec5: [F32;1024],
	fRec16: [F32;2],
	fRec27: [F32;2],
	fRec29: [F32;2],
	fRec30: [F32;2],
	fVec6: [F32;2],
	fRec28: [F32;2],
	fRec31: [F32;2],
	fVec7: [F32;131072],
	fVec8: [F32;65536],
	fVec9: [F32;2],
	fRec33: [F32;2],
	fRec32: [F32;2],
	fVec10: [F32;1024],
	fRec25: [F32;2],
	fRec37: [F32;2],
	fRec38: [F32;2],
	fVec11: [F32;2],
	fRec36: [F32;2],
	fRec39: [F32;2],
	fVec12: [F32;131072],
	fVec13: [F32;1024],
	fRec34: [F32;2],
	fRec42: [F32;2],
	fRec44: [F32;2],
	fRec45: [F32;2],
	fVec14: [F32;2],
	fRec43: [F32;2],
	fRec46: [F32;2],
	fVec15: [F32;131072],
	fVec16: [F32;1024],
	fRec40: [F32;2],
	fRec50: [F32;2],
	fRec51: [F32;2],
	fVec17: [F32;2],
	fRec49: [F32;2],
	fRec52: [F32;2],
	fVec18: [F32;131072],
	fVec19: [F32;1024],
	fRec47: [F32;2],
	fRec55: [F32;2],
	fRec57: [F32;2],
	fRec58: [F32;2],
	fVec20: [F32;2],
	fRec56: [F32;2],
	fRec59: [F32;2],
	fVec21: [F32;131072],
	fVec22: [F32;1024],
	fRec53: [F32;2],
	fRec63: [F32;2],
	fRec64: [F32;2],
	fVec23: [F32;2],
	fRec62: [F32;2],
	fRec65: [F32;2],
	fVec24: [F32;65536],
	fVec25: [F32;1024],
	fRec60: [F32;2],
	fRec68: [F32;2],
	fRec70: [F32;2],
	fRec71: [F32;2],
	fVec26: [F32;2],
	fRec69: [F32;2],
	fRec72: [F32;2],
	fVec27: [F32;131072],
	fVec28: [F32;1024],
	fRec66: [F32;2],
	fRec76: [F32;2],
	fRec77: [F32;2],
	fVec29: [F32;2],
	fRec75: [F32;2],
	fRec78: [F32;2],
	fVec30: [F32;131072],
	fVec31: [F32;1024],
	fRec73: [F32;2],
	fRec81: [F32;2],
	fRec83: [F32;2],
	fRec84: [F32;2],
	fVec32: [F32;2],
	fRec82: [F32;2],
	fRec85: [F32;2],
	fVec33: [F32;131072],
	fVec34: [F32;1024],
	fRec79: [F32;2],
	fRec89: [F32;2],
	fRec90: [F32;2],
	fVec35: [F32;2],
	fRec88: [F32;2],
	fRec91: [F32;2],
	fVec36: [F32;65536],
	fVec37: [F32;1024],
	fRec86: [F32;2],
	fRec94: [F32;2],
	fRec96: [F32;2],
	fRec97: [F32;2],
	fVec38: [F32;2],
	fRec95: [F32;2],
	fRec98: [F32;2],
	fVec39: [F32;131072],
	fVec40: [F32;1024],
	fRec92: [F32;2],
	fRec102: [F32;2],
	fRec103: [F32;2],
	fVec41: [F32;2],
	fRec101: [F32;2],
	fRec104: [F32;2],
	fVec42: [F32;131072],
	fVec43: [F32;1024],
	fRec99: [F32;2],
	fRec107: [F32;2],
	fRec109: [F32;2],
	fRec110: [F32;2],
	fVec44: [F32;2],
	fRec108: [F32;2],
	fRec111: [F32;2],
	fVec45: [F32;65536],
	fVec46: [F32;1024],
	fRec105: [F32;2],
	fRec115: [F32;2],
	fRec116: [F32;2],
	fVec47: [F32;2],
	fRec114: [F32;2],
	fRec117: [F32;2],
	fVec48: [F32;65536],
	fVec49: [F32;1024],
	fRec112: [F32;2],
	fRec121: [F32;2],
	fRec122: [F32;2],
	fVec50: [F32;2],
	fRec120: [F32;2],
	fRec123: [F32;2],
	fVec51: [F32;131072],
	fVec52: [F32;1024],
	fRec118: [F32;2],
	fRec0: [F32;3],
	fRec1: [F32;3],
	fRec2: [F32;3],
	fRec3: [F32;3],
	fRec4: [F32;3],
	fRec5: [F32;3],
	fRec6: [F32;3],
	fRec7: [F32;3],
	fRec8: [F32;3],
	fRec9: [F32;3],
	fRec10: [F32;3],
	fRec11: [F32;3],
	fRec12: [F32;3],
	fRec13: [F32;3],
	fRec14: [F32;3],
	fRec15: [F32;3],
}

pub type FaustFloat = F32;
fn VitalRevDsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
fn remainder_f32(from: f32, to: f32) -> f32 {
	from - to * (from / to).round_ties_even()
}
fn rint_f32(val: f32) -> f32 {
	val.round_ties_even()
}

pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 11;
pub const FAUST_PASSIVES: usize = 0;


impl VitalRevDsp {
		
	pub fn new() -> VitalRevDsp { 
		VitalRevDsp {
			fSampleRate: 0,
			fConst0: 0.0,
			fHslider0: 0.0,
			fHslider1: 0.0,
			iVec0: [0;2],
			fConst1: 0.0,
			fHslider2: 0.0,
			fRec18: [0.0;2],
			fHslider3: 0.0,
			fConst2: 0.0,
			fHslider4: 0.0,
			fHslider5: 0.0,
			fRec20: [0.0;2],
			fHslider6: 0.0,
			fRec21: [0.0;2],
			fVec1: [0.0;2],
			fRec19: [0.0;2],
			fHslider7: 0.0,
			fRec22: [0.0;2],
			IOTA0: 0,
			fVec2: [0.0;131072],
			fConst3: 0.0,
			fHslider8: 0.0,
			fHslider9: 0.0,
			fConst4: 0.0,
			fHslider10: 0.0,
			fVec3: [0.0;65536],
			fVec4: [0.0;2],
			fRec24: [0.0;2],
			fRec23: [0.0;2],
			fVec5: [0.0;1024],
			fRec16: [0.0;2],
			fRec27: [0.0;2],
			fRec29: [0.0;2],
			fRec30: [0.0;2],
			fVec6: [0.0;2],
			fRec28: [0.0;2],
			fRec31: [0.0;2],
			fVec7: [0.0;131072],
			fVec8: [0.0;65536],
			fVec9: [0.0;2],
			fRec33: [0.0;2],
			fRec32: [0.0;2],
			fVec10: [0.0;1024],
			fRec25: [0.0;2],
			fRec37: [0.0;2],
			fRec38: [0.0;2],
			fVec11: [0.0;2],
			fRec36: [0.0;2],
			fRec39: [0.0;2],
			fVec12: [0.0;131072],
			fVec13: [0.0;1024],
			fRec34: [0.0;2],
			fRec42: [0.0;2],
			fRec44: [0.0;2],
			fRec45: [0.0;2],
			fVec14: [0.0;2],
			fRec43: [0.0;2],
			fRec46: [0.0;2],
			fVec15: [0.0;131072],
			fVec16: [0.0;1024],
			fRec40: [0.0;2],
			fRec50: [0.0;2],
			fRec51: [0.0;2],
			fVec17: [0.0;2],
			fRec49: [0.0;2],
			fRec52: [0.0;2],
			fVec18: [0.0;131072],
			fVec19: [0.0;1024],
			fRec47: [0.0;2],
			fRec55: [0.0;2],
			fRec57: [0.0;2],
			fRec58: [0.0;2],
			fVec20: [0.0;2],
			fRec56: [0.0;2],
			fRec59: [0.0;2],
			fVec21: [0.0;131072],
			fVec22: [0.0;1024],
			fRec53: [0.0;2],
			fRec63: [0.0;2],
			fRec64: [0.0;2],
			fVec23: [0.0;2],
			fRec62: [0.0;2],
			fRec65: [0.0;2],
			fVec24: [0.0;65536],
			fVec25: [0.0;1024],
			fRec60: [0.0;2],
			fRec68: [0.0;2],
			fRec70: [0.0;2],
			fRec71: [0.0;2],
			fVec26: [0.0;2],
			fRec69: [0.0;2],
			fRec72: [0.0;2],
			fVec27: [0.0;131072],
			fVec28: [0.0;1024],
			fRec66: [0.0;2],
			fRec76: [0.0;2],
			fRec77: [0.0;2],
			fVec29: [0.0;2],
			fRec75: [0.0;2],
			fRec78: [0.0;2],
			fVec30: [0.0;131072],
			fVec31: [0.0;1024],
			fRec73: [0.0;2],
			fRec81: [0.0;2],
			fRec83: [0.0;2],
			fRec84: [0.0;2],
			fVec32: [0.0;2],
			fRec82: [0.0;2],
			fRec85: [0.0;2],
			fVec33: [0.0;131072],
			fVec34: [0.0;1024],
			fRec79: [0.0;2],
			fRec89: [0.0;2],
			fRec90: [0.0;2],
			fVec35: [0.0;2],
			fRec88: [0.0;2],
			fRec91: [0.0;2],
			fVec36: [0.0;65536],
			fVec37: [0.0;1024],
			fRec86: [0.0;2],
			fRec94: [0.0;2],
			fRec96: [0.0;2],
			fRec97: [0.0;2],
			fVec38: [0.0;2],
			fRec95: [0.0;2],
			fRec98: [0.0;2],
			fVec39: [0.0;131072],
			fVec40: [0.0;1024],
			fRec92: [0.0;2],
			fRec102: [0.0;2],
			fRec103: [0.0;2],
			fVec41: [0.0;2],
			fRec101: [0.0;2],
			fRec104: [0.0;2],
			fVec42: [0.0;131072],
			fVec43: [0.0;1024],
			fRec99: [0.0;2],
			fRec107: [0.0;2],
			fRec109: [0.0;2],
			fRec110: [0.0;2],
			fVec44: [0.0;2],
			fRec108: [0.0;2],
			fRec111: [0.0;2],
			fVec45: [0.0;65536],
			fVec46: [0.0;1024],
			fRec105: [0.0;2],
			fRec115: [0.0;2],
			fRec116: [0.0;2],
			fVec47: [0.0;2],
			fRec114: [0.0;2],
			fRec117: [0.0;2],
			fVec48: [0.0;65536],
			fVec49: [0.0;1024],
			fRec112: [0.0;2],
			fRec121: [0.0;2],
			fRec122: [0.0;2],
			fVec50: [0.0;2],
			fRec120: [0.0;2],
			fRec123: [0.0;2],
			fVec51: [0.0;131072],
			fVec52: [0.0;1024],
			fRec118: [0.0;2],
			fRec0: [0.0;3],
			fRec1: [0.0;3],
			fRec2: [0.0;3],
			fRec3: [0.0;3],
			fRec4: [0.0;3],
			fRec5: [0.0;3],
			fRec6: [0.0;3],
			fRec7: [0.0;3],
			fRec8: [0.0;3],
			fRec9: [0.0;3],
			fRec10: [0.0;3],
			fRec11: [0.0;3],
			fRec12: [0.0;3],
			fRec13: [0.0;3],
			fRec14: [0.0;3],
			fRec15: [0.0;3],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("aanl.lib/name", r"Faust Antialiased Nonlinearities");
		m.declare("aanl.lib/version", r"1.4.1");
		m.declare("analyzers.lib/name", r"Faust Analyzer Library");
		m.declare("analyzers.lib/version", r"1.2.0");
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/version", r"1.21.0");
		m.declare("compile_options", r"-lang rust -ct 1 -cn VitalRevDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("delays.lib/fdelayltv:author", r"Julius O. Smith III");
		m.declare("delays.lib/name", r"Faust Delay Library");
		m.declare("delays.lib/version", r"1.2.0");
		m.declare("filename", r"vital_rev.dsp");
		m.declare("filters.lib/allpass_comb:author", r"Julius O. Smith III");
		m.declare("filters.lib/allpass_comb:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/allpass_comb:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/filterbank:author", r"Julius O. Smith III");
		m.declare("filters.lib/filterbank:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/filterbank:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/highpass:author", r"Julius O. Smith III");
		m.declare("filters.lib/highpass:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/highshelf:author", r"Julius O. Smith III");
		m.declare("filters.lib/highshelf:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/highshelf:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/lowpass0_highpass1", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/lowpass0_highpass1:author", r"Julius O. Smith III");
		m.declare("filters.lib/lowpass:author", r"Julius O. Smith III");
		m.declare("filters.lib/lowpass:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/lowpass:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/lowshelf:author", r"Julius O. Smith III");
		m.declare("filters.lib/lowshelf:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/lowshelf:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/name", r"Faust Filters Library");
		m.declare("filters.lib/tf1:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf1:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf1:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/tf1s:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf1s:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf1s:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/version", r"1.7.1");
		m.declare("interpolators.lib/interpolate_linear:author", r"Stéphane Letz");
		m.declare("interpolators.lib/interpolate_linear:licence", r"MIT");
		m.declare("interpolators.lib/name", r"Faust Interpolator Library");
		m.declare("interpolators.lib/remap:author", r"David Braun");
		m.declare("interpolators.lib/version", r"1.4.0");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("misceffects.lib/dryWetMixerConstantPower:author", r"David Braun, revised by Stéphane Letz");
		m.declare("misceffects.lib/name", r"Misc Effects Library");
		m.declare("misceffects.lib/version", r"2.5.1");
		m.declare("name", r"vital_rev");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
		m.declare("reverbs.lib/name", r"Faust Reverb Library");
		m.declare("reverbs.lib/version", r"1.4.0");
		m.declare("reverbs.lib/vital_rev:author", r"David Braun");
		m.declare("reverbs.lib/vital_rev:license", r"GPL-3.0");
		m.declare("routes.lib/name", r"Faust Signal Routing Library");
		m.declare("routes.lib/version", r"1.2.0");
		m.declare("signals.lib/name", r"Faust Signal Routing Library");
		m.declare("signals.lib/version", r"1.6.0");
		m.declare("spats.lib/name", r"Faust Spatialization Library");
		m.declare("spats.lib/version", r"1.2.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.75;
		self.fHslider1 = 0.3;
		self.fHslider2 = 0.65;
		self.fHslider3 = 0.55;
		self.fHslider4 = 0.7;
		self.fHslider5 = 0.5;
		self.fHslider6 = 0.4;
		self.fHslider7 = 0.5;
		self.fHslider8 = 0.9;
		self.fHslider9 = 0.2;
		self.fHslider10 = 0.0;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.iVec0[l0 as usize] = 0;
		}
		for l1 in 0..2 {
			self.fRec18[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec20[l2 as usize] = 0.0;
		}
		for l3 in 0..2 {
			self.fRec21[l3 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fVec1[l4 as usize] = 0.0;
		}
		for l5 in 0..2 {
			self.fRec19[l5 as usize] = 0.0;
		}
		for l6 in 0..2 {
			self.fRec22[l6 as usize] = 0.0;
		}
		self.IOTA0 = 0;
		for l7 in 0..131072 {
			self.fVec2[l7 as usize] = 0.0;
		}
		for l8 in 0..65536 {
			self.fVec3[l8 as usize] = 0.0;
		}
		for l9 in 0..2 {
			self.fVec4[l9 as usize] = 0.0;
		}
		for l10 in 0..2 {
			self.fRec24[l10 as usize] = 0.0;
		}
		for l11 in 0..2 {
			self.fRec23[l11 as usize] = 0.0;
		}
		for l12 in 0..1024 {
			self.fVec5[l12 as usize] = 0.0;
		}
		for l13 in 0..2 {
			self.fRec16[l13 as usize] = 0.0;
		}
		for l14 in 0..2 {
			self.fRec27[l14 as usize] = 0.0;
		}
		for l15 in 0..2 {
			self.fRec29[l15 as usize] = 0.0;
		}
		for l16 in 0..2 {
			self.fRec30[l16 as usize] = 0.0;
		}
		for l17 in 0..2 {
			self.fVec6[l17 as usize] = 0.0;
		}
		for l18 in 0..2 {
			self.fRec28[l18 as usize] = 0.0;
		}
		for l19 in 0..2 {
			self.fRec31[l19 as usize] = 0.0;
		}
		for l20 in 0..131072 {
			self.fVec7[l20 as usize] = 0.0;
		}
		for l21 in 0..65536 {
			self.fVec8[l21 as usize] = 0.0;
		}
		for l22 in 0..2 {
			self.fVec9[l22 as usize] = 0.0;
		}
		for l23 in 0..2 {
			self.fRec33[l23 as usize] = 0.0;
		}
		for l24 in 0..2 {
			self.fRec32[l24 as usize] = 0.0;
		}
		for l25 in 0..1024 {
			self.fVec10[l25 as usize] = 0.0;
		}
		for l26 in 0..2 {
			self.fRec25[l26 as usize] = 0.0;
		}
		for l27 in 0..2 {
			self.fRec37[l27 as usize] = 0.0;
		}
		for l28 in 0..2 {
			self.fRec38[l28 as usize] = 0.0;
		}
		for l29 in 0..2 {
			self.fVec11[l29 as usize] = 0.0;
		}
		for l30 in 0..2 {
			self.fRec36[l30 as usize] = 0.0;
		}
		for l31 in 0..2 {
			self.fRec39[l31 as usize] = 0.0;
		}
		for l32 in 0..131072 {
			self.fVec12[l32 as usize] = 0.0;
		}
		for l33 in 0..1024 {
			self.fVec13[l33 as usize] = 0.0;
		}
		for l34 in 0..2 {
			self.fRec34[l34 as usize] = 0.0;
		}
		for l35 in 0..2 {
			self.fRec42[l35 as usize] = 0.0;
		}
		for l36 in 0..2 {
			self.fRec44[l36 as usize] = 0.0;
		}
		for l37 in 0..2 {
			self.fRec45[l37 as usize] = 0.0;
		}
		for l38 in 0..2 {
			self.fVec14[l38 as usize] = 0.0;
		}
		for l39 in 0..2 {
			self.fRec43[l39 as usize] = 0.0;
		}
		for l40 in 0..2 {
			self.fRec46[l40 as usize] = 0.0;
		}
		for l41 in 0..131072 {
			self.fVec15[l41 as usize] = 0.0;
		}
		for l42 in 0..1024 {
			self.fVec16[l42 as usize] = 0.0;
		}
		for l43 in 0..2 {
			self.fRec40[l43 as usize] = 0.0;
		}
		for l44 in 0..2 {
			self.fRec50[l44 as usize] = 0.0;
		}
		for l45 in 0..2 {
			self.fRec51[l45 as usize] = 0.0;
		}
		for l46 in 0..2 {
			self.fVec17[l46 as usize] = 0.0;
		}
		for l47 in 0..2 {
			self.fRec49[l47 as usize] = 0.0;
		}
		for l48 in 0..2 {
			self.fRec52[l48 as usize] = 0.0;
		}
		for l49 in 0..131072 {
			self.fVec18[l49 as usize] = 0.0;
		}
		for l50 in 0..1024 {
			self.fVec19[l50 as usize] = 0.0;
		}
		for l51 in 0..2 {
			self.fRec47[l51 as usize] = 0.0;
		}
		for l52 in 0..2 {
			self.fRec55[l52 as usize] = 0.0;
		}
		for l53 in 0..2 {
			self.fRec57[l53 as usize] = 0.0;
		}
		for l54 in 0..2 {
			self.fRec58[l54 as usize] = 0.0;
		}
		for l55 in 0..2 {
			self.fVec20[l55 as usize] = 0.0;
		}
		for l56 in 0..2 {
			self.fRec56[l56 as usize] = 0.0;
		}
		for l57 in 0..2 {
			self.fRec59[l57 as usize] = 0.0;
		}
		for l58 in 0..131072 {
			self.fVec21[l58 as usize] = 0.0;
		}
		for l59 in 0..1024 {
			self.fVec22[l59 as usize] = 0.0;
		}
		for l60 in 0..2 {
			self.fRec53[l60 as usize] = 0.0;
		}
		for l61 in 0..2 {
			self.fRec63[l61 as usize] = 0.0;
		}
		for l62 in 0..2 {
			self.fRec64[l62 as usize] = 0.0;
		}
		for l63 in 0..2 {
			self.fVec23[l63 as usize] = 0.0;
		}
		for l64 in 0..2 {
			self.fRec62[l64 as usize] = 0.0;
		}
		for l65 in 0..2 {
			self.fRec65[l65 as usize] = 0.0;
		}
		for l66 in 0..65536 {
			self.fVec24[l66 as usize] = 0.0;
		}
		for l67 in 0..1024 {
			self.fVec25[l67 as usize] = 0.0;
		}
		for l68 in 0..2 {
			self.fRec60[l68 as usize] = 0.0;
		}
		for l69 in 0..2 {
			self.fRec68[l69 as usize] = 0.0;
		}
		for l70 in 0..2 {
			self.fRec70[l70 as usize] = 0.0;
		}
		for l71 in 0..2 {
			self.fRec71[l71 as usize] = 0.0;
		}
		for l72 in 0..2 {
			self.fVec26[l72 as usize] = 0.0;
		}
		for l73 in 0..2 {
			self.fRec69[l73 as usize] = 0.0;
		}
		for l74 in 0..2 {
			self.fRec72[l74 as usize] = 0.0;
		}
		for l75 in 0..131072 {
			self.fVec27[l75 as usize] = 0.0;
		}
		for l76 in 0..1024 {
			self.fVec28[l76 as usize] = 0.0;
		}
		for l77 in 0..2 {
			self.fRec66[l77 as usize] = 0.0;
		}
		for l78 in 0..2 {
			self.fRec76[l78 as usize] = 0.0;
		}
		for l79 in 0..2 {
			self.fRec77[l79 as usize] = 0.0;
		}
		for l80 in 0..2 {
			self.fVec29[l80 as usize] = 0.0;
		}
		for l81 in 0..2 {
			self.fRec75[l81 as usize] = 0.0;
		}
		for l82 in 0..2 {
			self.fRec78[l82 as usize] = 0.0;
		}
		for l83 in 0..131072 {
			self.fVec30[l83 as usize] = 0.0;
		}
		for l84 in 0..1024 {
			self.fVec31[l84 as usize] = 0.0;
		}
		for l85 in 0..2 {
			self.fRec73[l85 as usize] = 0.0;
		}
		for l86 in 0..2 {
			self.fRec81[l86 as usize] = 0.0;
		}
		for l87 in 0..2 {
			self.fRec83[l87 as usize] = 0.0;
		}
		for l88 in 0..2 {
			self.fRec84[l88 as usize] = 0.0;
		}
		for l89 in 0..2 {
			self.fVec32[l89 as usize] = 0.0;
		}
		for l90 in 0..2 {
			self.fRec82[l90 as usize] = 0.0;
		}
		for l91 in 0..2 {
			self.fRec85[l91 as usize] = 0.0;
		}
		for l92 in 0..131072 {
			self.fVec33[l92 as usize] = 0.0;
		}
		for l93 in 0..1024 {
			self.fVec34[l93 as usize] = 0.0;
		}
		for l94 in 0..2 {
			self.fRec79[l94 as usize] = 0.0;
		}
		for l95 in 0..2 {
			self.fRec89[l95 as usize] = 0.0;
		}
		for l96 in 0..2 {
			self.fRec90[l96 as usize] = 0.0;
		}
		for l97 in 0..2 {
			self.fVec35[l97 as usize] = 0.0;
		}
		for l98 in 0..2 {
			self.fRec88[l98 as usize] = 0.0;
		}
		for l99 in 0..2 {
			self.fRec91[l99 as usize] = 0.0;
		}
		for l100 in 0..65536 {
			self.fVec36[l100 as usize] = 0.0;
		}
		for l101 in 0..1024 {
			self.fVec37[l101 as usize] = 0.0;
		}
		for l102 in 0..2 {
			self.fRec86[l102 as usize] = 0.0;
		}
		for l103 in 0..2 {
			self.fRec94[l103 as usize] = 0.0;
		}
		for l104 in 0..2 {
			self.fRec96[l104 as usize] = 0.0;
		}
		for l105 in 0..2 {
			self.fRec97[l105 as usize] = 0.0;
		}
		for l106 in 0..2 {
			self.fVec38[l106 as usize] = 0.0;
		}
		for l107 in 0..2 {
			self.fRec95[l107 as usize] = 0.0;
		}
		for l108 in 0..2 {
			self.fRec98[l108 as usize] = 0.0;
		}
		for l109 in 0..131072 {
			self.fVec39[l109 as usize] = 0.0;
		}
		for l110 in 0..1024 {
			self.fVec40[l110 as usize] = 0.0;
		}
		for l111 in 0..2 {
			self.fRec92[l111 as usize] = 0.0;
		}
		for l112 in 0..2 {
			self.fRec102[l112 as usize] = 0.0;
		}
		for l113 in 0..2 {
			self.fRec103[l113 as usize] = 0.0;
		}
		for l114 in 0..2 {
			self.fVec41[l114 as usize] = 0.0;
		}
		for l115 in 0..2 {
			self.fRec101[l115 as usize] = 0.0;
		}
		for l116 in 0..2 {
			self.fRec104[l116 as usize] = 0.0;
		}
		for l117 in 0..131072 {
			self.fVec42[l117 as usize] = 0.0;
		}
		for l118 in 0..1024 {
			self.fVec43[l118 as usize] = 0.0;
		}
		for l119 in 0..2 {
			self.fRec99[l119 as usize] = 0.0;
		}
		for l120 in 0..2 {
			self.fRec107[l120 as usize] = 0.0;
		}
		for l121 in 0..2 {
			self.fRec109[l121 as usize] = 0.0;
		}
		for l122 in 0..2 {
			self.fRec110[l122 as usize] = 0.0;
		}
		for l123 in 0..2 {
			self.fVec44[l123 as usize] = 0.0;
		}
		for l124 in 0..2 {
			self.fRec108[l124 as usize] = 0.0;
		}
		for l125 in 0..2 {
			self.fRec111[l125 as usize] = 0.0;
		}
		for l126 in 0..65536 {
			self.fVec45[l126 as usize] = 0.0;
		}
		for l127 in 0..1024 {
			self.fVec46[l127 as usize] = 0.0;
		}
		for l128 in 0..2 {
			self.fRec105[l128 as usize] = 0.0;
		}
		for l129 in 0..2 {
			self.fRec115[l129 as usize] = 0.0;
		}
		for l130 in 0..2 {
			self.fRec116[l130 as usize] = 0.0;
		}
		for l131 in 0..2 {
			self.fVec47[l131 as usize] = 0.0;
		}
		for l132 in 0..2 {
			self.fRec114[l132 as usize] = 0.0;
		}
		for l133 in 0..2 {
			self.fRec117[l133 as usize] = 0.0;
		}
		for l134 in 0..65536 {
			self.fVec48[l134 as usize] = 0.0;
		}
		for l135 in 0..1024 {
			self.fVec49[l135 as usize] = 0.0;
		}
		for l136 in 0..2 {
			self.fRec112[l136 as usize] = 0.0;
		}
		for l137 in 0..2 {
			self.fRec121[l137 as usize] = 0.0;
		}
		for l138 in 0..2 {
			self.fRec122[l138 as usize] = 0.0;
		}
		for l139 in 0..2 {
			self.fVec50[l139 as usize] = 0.0;
		}
		for l140 in 0..2 {
			self.fRec120[l140 as usize] = 0.0;
		}
		for l141 in 0..2 {
			self.fRec123[l141 as usize] = 0.0;
		}
		for l142 in 0..131072 {
			self.fVec51[l142 as usize] = 0.0;
		}
		for l143 in 0..1024 {
			self.fVec52[l143 as usize] = 0.0;
		}
		for l144 in 0..2 {
			self.fRec118[l144 as usize] = 0.0;
		}
		for l145 in 0..3 {
			self.fRec0[l145 as usize] = 0.0;
		}
		for l146 in 0..3 {
			self.fRec1[l146 as usize] = 0.0;
		}
		for l147 in 0..3 {
			self.fRec2[l147 as usize] = 0.0;
		}
		for l148 in 0..3 {
			self.fRec3[l148 as usize] = 0.0;
		}
		for l149 in 0..3 {
			self.fRec4[l149 as usize] = 0.0;
		}
		for l150 in 0..3 {
			self.fRec5[l150 as usize] = 0.0;
		}
		for l151 in 0..3 {
			self.fRec6[l151 as usize] = 0.0;
		}
		for l152 in 0..3 {
			self.fRec7[l152 as usize] = 0.0;
		}
		for l153 in 0..3 {
			self.fRec8[l153 as usize] = 0.0;
		}
		for l154 in 0..3 {
			self.fRec9[l154 as usize] = 0.0;
		}
		for l155 in 0..3 {
			self.fRec10[l155 as usize] = 0.0;
		}
		for l156 in 0..3 {
			self.fRec11[l156 as usize] = 0.0;
		}
		for l157 in 0..3 {
			self.fRec12[l157 as usize] = 0.0;
		}
		for l158 in 0..3 {
			self.fRec13[l158 as usize] = 0.0;
		}
		for l159 in 0..3 {
			self.fRec14[l159 as usize] = 0.0;
		}
		for l160 in 0..3 {
			self.fRec15[l160 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = 1.0 / self.fConst0;
		self.fConst2 = 1382.3008 / self.fConst0;
		self.fConst3 = 0.62716556 * self.fConst0;
		self.fConst4 = 0.3 * self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		VitalRevDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("vital_rev");
		ui_interface.add_horizontal_slider("a_prelow", ParamIndex(0), 0.2, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("b_prehigh", ParamIndex(1), 0.9, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("c_lowcut", ParamIndex(2), 0.5, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("d_highcut", ParamIndex(3), 0.7, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("e_lowgain", ParamIndex(4), 0.4, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("f_highgain", ParamIndex(5), 0.5, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("g_chorus", ParamIndex(6), 0.3, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("h_chorusfreq", ParamIndex(7), 0.65, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("i_predelay", ParamIndex(8), 0.0, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("j_time", ParamIndex(9), 0.55, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("k_size", ParamIndex(10), 0.75, 0.0, 1.0, 0.001);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			10 => Some(self.fHslider0),
			6 => Some(self.fHslider1),
			8 => Some(self.fHslider10),
			7 => Some(self.fHslider2),
			9 => Some(self.fHslider3),
			3 => Some(self.fHslider4),
			2 => Some(self.fHslider5),
			4 => Some(self.fHslider6),
			5 => Some(self.fHslider7),
			1 => Some(self.fHslider8),
			0 => Some(self.fHslider9),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			10 => { self.fHslider0 = value }
			6 => { self.fHslider1 = value }
			8 => { self.fHslider10 = value }
			7 => { self.fHslider2 = value }
			9 => { self.fHslider3 = value }
			3 => { self.fHslider4 = value }
			2 => { self.fHslider5 = value }
			4 => { self.fHslider6 = value }
			5 => { self.fHslider7 = value }
			1 => { self.fHslider8 = value }
			0 => { self.fHslider9 = value }
			_ => {}
		}
	}
	
	pub fn compute(
		&mut self,
		count: usize,
		inputs: &[impl AsRef<[FaustFloat]>],
		outputs: &mut[impl AsMut<[FaustFloat]>],
	) {
		
		// Obtaining locks on 0 static var(s)
		let [inputs0, inputs1, .. ] = inputs.as_ref() else { panic!("wrong number of input buffers"); };
		let inputs0 = inputs0.as_ref()[..count].iter();
		let inputs1 = inputs1.as_ref()[..count].iter();
		let [outputs0, outputs1, .. ] = outputs.as_mut() else { panic!("wrong number of output buffers"); };
		let outputs0 = outputs0.as_mut()[..count].iter_mut();
		let outputs1 = outputs1.as_mut()[..count].iter_mut();
		let mut fSlow0: F32 = F32::powf(2.0, 4.0 * F32::max(0.0, F32::min(1.0, self.fHslider0)) + -3.0);
		let mut fSlow1: F32 = self.fConst0 * fSlow0;
		let mut fSlow2: F32 = 0.05668934 * VitalRevDsp_faustpower2_f(F32::max(0.0, F32::min(1.0, self.fHslider1)));
		let mut fSlow3: F32 = self.fConst1 * F32::min(16.0, F32::exp(11.0 * self.fHslider2 + -8.0));
		let mut fSlow4: F32 = fSlow0 / F32::max(0.1, F32::min(1e+02, F32::exp(12.0 * self.fHslider3 + -6.0)));
		let mut fSlow5: F32 = F32::powf(0.001, 0.15313378 * fSlow4);
		let mut fSlow6: F32 = 1.0 / F32::tan(self.fConst2 * F32::powf(2.0, 0.083333336 * (119.0 * F32::max(0.0, F32::min(1.0, self.fHslider4)) + -53.0)));
		let mut fSlow7: F32 = 1.0 / (fSlow6 + 1.0);
		let mut fSlow8: F32 = 1.0 - fSlow6;
		let mut fSlow9: F32 = 1.0 / F32::tan(self.fConst2 * F32::powf(2.0, 0.083333336 * (119.0 * F32::max(0.0, F32::min(1.0, self.fHslider5)) + -53.0)));
		let mut fSlow10: F32 = 1.0 / (fSlow9 + 1.0);
		let mut fSlow11: F32 = 1.0 - fSlow9;
		let mut fSlow12: F32 = F32::powf(1e+01, -(1.2 * (1.0 - F32::max(0.0, F32::min(1.0, self.fHslider6)))));
		let mut fSlow13: F32 = F32::powf(1e+01, -(1.2 * (1.0 - F32::max(0.0, F32::min(1.0, self.fHslider7)))));
		let mut fSlow14: F32 = 1.0 / F32::tan(self.fConst2 * F32::powf(2.0, 0.083333336 * (119.0 * F32::max(0.0, F32::min(1.0, self.fHslider8)) + -53.0)));
		let mut fSlow15: F32 = 1.0 / (fSlow14 + 1.0);
		let mut fSlow16: F32 = 1.0 - fSlow14;
		let mut fSlow17: F32 = 1.0 / F32::tan(self.fConst2 * F32::powf(2.0, 0.083333336 * (119.0 * F32::max(0.0, F32::min(1.0, self.fHslider9)) + -53.0)));
		let mut fSlow18: F32 = 1.0 / (fSlow17 + 1.0);
		let mut fSlow19: F32 = 1.0 - fSlow17;
		let mut fSlow20: F32 = F32::max(1.0, self.fConst4 * F32::max(0.0, F32::min(1.0, self.fHslider10)));
		let mut fSlow21: F32 = fSlow20 + -0.999995;
		let mut fSlow22: F32 = F32::floor(fSlow21);
		let mut fSlow23: F32 = fSlow20 + (-3.0 - fSlow22);
		let mut fSlow24: F32 = fSlow20 + (-2.0 - fSlow22);
		let mut fSlow25: F32 = fSlow20 - fSlow22;
		let mut fSlow26: F32 = 0.5 * fSlow25;
		let mut iSlow27: i32 = (fSlow21) as i32;
		let mut iSlow28: i32 = (F32::min(self.fConst4, (std::cmp::max(0, i32::wrapping_add(iSlow27, 1))) as F32)) as i32;
		let mut fSlow29: F32 = fSlow20 + (-1.0 - fSlow22);
		let mut fSlow30: F32 = 0.16666667 * fSlow29;
		let mut iSlow31: i32 = (F32::min(self.fConst4, (std::cmp::max(0, iSlow27)) as F32)) as i32;
		let mut fSlow32: F32 = fSlow25 * fSlow29;
		let mut fSlow33: F32 = 0.5 * fSlow32;
		let mut iSlow34: i32 = (F32::min(self.fConst4, (std::cmp::max(0, i32::wrapping_add(iSlow27, 2))) as F32)) as i32;
		let mut fSlow35: F32 = 0.16666667 * fSlow32 * fSlow24;
		let mut iSlow36: i32 = (F32::min(self.fConst4, (std::cmp::max(0, i32::wrapping_add(iSlow27, 3))) as F32)) as i32;
		let mut fSlow37: F32 = F32::powf(0.001, 0.12767006 * fSlow4);
		let mut fSlow38: F32 = F32::powf(0.001, 0.17490976 * fSlow4);
		let mut fSlow39: F32 = F32::powf(0.001, 0.12786055 * fSlow4);
		let mut fSlow40: F32 = F32::powf(0.001, 0.25688207 * fSlow4);
		let mut fSlow41: F32 = F32::powf(0.001, 0.11940046 * fSlow4);
		let mut fSlow42: F32 = F32::powf(0.001, 0.08223061 * fSlow4);
		let mut fSlow43: F32 = F32::powf(0.001, 0.19230045 * fSlow4);
		let mut fSlow44: F32 = F32::powf(0.001, 0.17470522 * fSlow4);
		let mut fSlow45: F32 = F32::powf(0.001, 0.14782245 * fSlow4);
		let mut fSlow46: F32 = F32::powf(0.001, 0.07776644 * fSlow4);
		let mut fSlow47: F32 = F32::powf(0.001, 0.125 * fSlow4);
		let mut fSlow48: F32 = F32::powf(0.001, 0.21039456 * fSlow4);
		let mut fSlow49: F32 = F32::powf(0.001, 0.10252925 * fSlow4);
		let mut fSlow50: F32 = F32::powf(0.001, 0.070764855 * fSlow4);
		let mut fSlow51: F32 = F32::powf(0.001, 0.21998005 * fSlow4);
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			self.iVec0[0] = 1;
			let mut iTemp0: i32 = i32::wrapping_sub(1, self.iVec0[1]);
			let mut fTemp1: F32 = (if iTemp0 != 0 {0.25} else {fSlow3 + self.fRec18[1]});
			self.fRec18[0] = fTemp1 - F32::floor(fTemp1);
			let mut fTemp2: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec18[0]);
			let mut fTemp3: F32 = F32::max(1.0, fSlow1 * (fTemp2 + 0.15313378));
			let mut fTemp4: F32 = fTemp3 + -0.999995;
			let mut fTemp5: F32 = F32::floor(fTemp4);
			let mut fTemp6: F32 = fTemp3 - fTemp5;
			let mut fTemp7: F32 = fTemp3 + (-1.0 - fTemp5);
			let mut fTemp8: F32 = fTemp6 * fTemp7;
			let mut fTemp9: F32 = fTemp3 + (-2.0 - fTemp5);
			self.fRec20[0] = -(fSlow10 * (fSlow11 * self.fRec20[1] - fSlow9 * (self.fRec0[1] - self.fRec0[2])));
			self.fRec21[0] = -(fSlow10 * (fSlow11 * self.fRec21[1] - (self.fRec0[1] + self.fRec0[2])));
			let mut fTemp10: F32 = self.fRec20[0] + fSlow12 * self.fRec21[0];
			self.fVec1[0] = fTemp10;
			self.fRec19[0] = -(fSlow7 * (fSlow8 * self.fRec19[1] - (fTemp10 + self.fVec1[1])));
			self.fRec22[0] = -(fSlow7 * (fSlow8 * self.fRec22[1] - fSlow6 * (fTemp10 - self.fVec1[1])));
			let mut fTemp11: F32 = fSlow5 * (self.fRec19[0] + fSlow13 * self.fRec22[0]);
			self.fVec2[(self.IOTA0 & 131071) as usize] = fTemp11;
			let mut iTemp12: i32 = (fTemp4) as i32;
			let mut fTemp13: F32 = *input0;
			self.fVec3[(self.IOTA0 & 65535) as usize] = fTemp13;
			let mut fTemp14: F32 = fSlow23 * (fSlow24 * (fSlow26 * self.fVec3[((i32::wrapping_sub(self.IOTA0, iSlow28)) & 65535) as usize] - fSlow30 * self.fVec3[((i32::wrapping_sub(self.IOTA0, iSlow31)) & 65535) as usize]) - fSlow33 * self.fVec3[((i32::wrapping_sub(self.IOTA0, iSlow34)) & 65535) as usize]) + fSlow35 * self.fVec3[((i32::wrapping_sub(self.IOTA0, iSlow36)) & 65535) as usize];
			self.fVec4[0] = fTemp14;
			self.fRec24[0] = -(fSlow18 * (fSlow19 * self.fRec24[1] - fSlow17 * (fTemp14 - self.fVec4[1])));
			self.fRec23[0] = -(fSlow15 * (fSlow16 * self.fRec23[1] - (self.fRec24[0] + self.fRec24[1])));
			let mut fTemp15: F32 = 0.25 * self.fRec23[0];
			let mut fTemp16: F32 = 0.16666667 * fTemp8 * fTemp9 * self.fVec2[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp12, 3))) as F32)) as i32)) & 131071) as usize] + (fTemp3 + (-3.0 - fTemp5)) * (fTemp9 * (0.5 * fTemp6 * self.fVec2[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp12, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec2[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp12)) as F32)) as i32)) & 131071) as usize] * fTemp7) - 0.5 * fTemp8 * self.fVec2[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp12, 2))) as F32)) as i32)) & 131071) as usize]) + fTemp15 - 0.6 * self.fRec16[1];
			self.fVec5[(self.IOTA0 & 1023) as usize] = fTemp16;
			self.fRec16[0] = self.fVec5[((i32::wrapping_sub(self.IOTA0, 1000)) & 1023) as usize];
			let mut fRec17: F32 = 0.6 * fTemp16;
			let mut fTemp17: F32 = (if iTemp0 != 0 {0.1875} else {fSlow3 + self.fRec27[1]});
			self.fRec27[0] = fTemp17 - F32::floor(fTemp17);
			let mut fTemp18: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec27[0]);
			let mut fTemp19: F32 = F32::max(1.0, fSlow1 * (0.12767006 - fTemp18));
			let mut fTemp20: F32 = fTemp19 + -0.999995;
			let mut fTemp21: F32 = F32::floor(fTemp20);
			let mut fTemp22: F32 = fTemp19 - fTemp21;
			let mut fTemp23: F32 = fTemp19 + (-1.0 - fTemp21);
			let mut fTemp24: F32 = fTemp22 * fTemp23;
			let mut fTemp25: F32 = fTemp19 + (-2.0 - fTemp21);
			self.fRec29[0] = -(fSlow10 * (fSlow11 * self.fRec29[1] - fSlow9 * (self.fRec15[1] - self.fRec15[2])));
			self.fRec30[0] = -(fSlow10 * (fSlow11 * self.fRec30[1] - (self.fRec15[1] + self.fRec15[2])));
			let mut fTemp26: F32 = self.fRec29[0] + fSlow12 * self.fRec30[0];
			self.fVec6[0] = fTemp26;
			self.fRec28[0] = -(fSlow7 * (fSlow8 * self.fRec28[1] - (fTemp26 + self.fVec6[1])));
			self.fRec31[0] = -(fSlow7 * (fSlow8 * self.fRec31[1] - fSlow6 * (fTemp26 - self.fVec6[1])));
			let mut fTemp27: F32 = fSlow37 * (self.fRec28[0] + fSlow13 * self.fRec31[0]);
			self.fVec7[(self.IOTA0 & 131071) as usize] = fTemp27;
			let mut iTemp28: i32 = (fTemp20) as i32;
			let mut fTemp29: F32 = *input1;
			self.fVec8[(self.IOTA0 & 65535) as usize] = fTemp29;
			let mut fTemp30: F32 = fSlow23 * (fSlow24 * (fSlow26 * self.fVec8[((i32::wrapping_sub(self.IOTA0, iSlow28)) & 65535) as usize] - fSlow30 * self.fVec8[((i32::wrapping_sub(self.IOTA0, iSlow31)) & 65535) as usize]) - fSlow33 * self.fVec8[((i32::wrapping_sub(self.IOTA0, iSlow34)) & 65535) as usize]) + fSlow35 * self.fVec8[((i32::wrapping_sub(self.IOTA0, iSlow36)) & 65535) as usize];
			self.fVec9[0] = fTemp30;
			self.fRec33[0] = -(fSlow18 * (fSlow19 * self.fRec33[1] - fSlow17 * (fTemp30 - self.fVec9[1])));
			self.fRec32[0] = -(fSlow15 * (fSlow16 * self.fRec32[1] - (self.fRec33[0] + self.fRec33[1])));
			let mut fTemp31: F32 = 0.25 * self.fRec32[0];
			let mut fTemp32: F32 = 0.16666667 * fTemp24 * fTemp25 * self.fVec7[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp28, 3))) as F32)) as i32)) & 131071) as usize] + fTemp31 + (fTemp19 + (-3.0 - fTemp21)) * (fTemp25 * (0.5 * fTemp22 * self.fVec7[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp28, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec7[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp28)) as F32)) as i32)) & 131071) as usize] * fTemp23) - 0.5 * fTemp24 * self.fVec7[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp28, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec25[1];
			self.fVec10[(self.IOTA0 & 1023) as usize] = fTemp32;
			self.fRec25[0] = self.fVec10[((i32::wrapping_sub(self.IOTA0, 996)) & 1023) as usize];
			let mut fRec26: F32 = 0.6 * fTemp32;
			let mut fTemp33: F32 = F32::max(1.0, fSlow1 * (fTemp18 + 0.17490976));
			let mut fTemp34: F32 = fTemp33 + -0.999995;
			let mut fTemp35: F32 = F32::floor(fTemp34);
			let mut fTemp36: F32 = fTemp33 - fTemp35;
			let mut fTemp37: F32 = fTemp33 + (-1.0 - fTemp35);
			let mut fTemp38: F32 = fTemp36 * fTemp37;
			let mut fTemp39: F32 = fTemp33 + (-2.0 - fTemp35);
			self.fRec37[0] = -(fSlow10 * (fSlow11 * self.fRec37[1] - fSlow9 * (self.fRec11[1] - self.fRec11[2])));
			self.fRec38[0] = -(fSlow10 * (fSlow11 * self.fRec38[1] - (self.fRec11[1] + self.fRec11[2])));
			let mut fTemp40: F32 = self.fRec37[0] + fSlow12 * self.fRec38[0];
			self.fVec11[0] = fTemp40;
			self.fRec36[0] = -(fSlow7 * (fSlow8 * self.fRec36[1] - (fTemp40 + self.fVec11[1])));
			self.fRec39[0] = -(fSlow7 * (fSlow8 * self.fRec39[1] - fSlow6 * (fTemp40 - self.fVec11[1])));
			let mut fTemp41: F32 = fSlow38 * (self.fRec36[0] + fSlow13 * self.fRec39[0]);
			self.fVec12[(self.IOTA0 & 131071) as usize] = fTemp41;
			let mut iTemp42: i32 = (fTemp34) as i32;
			let mut fTemp43: F32 = 0.16666667 * fTemp38 * fTemp39 * self.fVec12[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp42, 3))) as F32)) as i32)) & 131071) as usize] + fTemp31 + (fTemp33 + (-3.0 - fTemp35)) * (fTemp39 * (0.5 * fTemp36 * self.fVec12[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp42, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec12[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp42)) as F32)) as i32)) & 131071) as usize] * fTemp37) - 0.5 * fTemp38 * self.fVec12[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp42, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec34[1];
			self.fVec13[(self.IOTA0 & 1023) as usize] = fTemp43;
			self.fRec34[0] = self.fVec13[((i32::wrapping_sub(self.IOTA0, 566)) & 1023) as usize];
			let mut fRec35: F32 = 0.6 * fTemp43;
			let mut fTemp44: F32 = (if iTemp0 != 0 {0.4375} else {fSlow3 + self.fRec42[1]});
			self.fRec42[0] = fTemp44 - F32::floor(fTemp44);
			let mut fTemp45: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec42[0]);
			let mut fTemp46: F32 = F32::max(1.0, fSlow1 * (0.12786055 - fTemp45));
			let mut fTemp47: F32 = fTemp46 + -0.999995;
			let mut fTemp48: F32 = F32::floor(fTemp47);
			let mut fTemp49: F32 = fTemp46 - fTemp48;
			let mut fTemp50: F32 = fTemp46 + (-1.0 - fTemp48);
			let mut fTemp51: F32 = fTemp49 * fTemp50;
			let mut fTemp52: F32 = fTemp46 + (-2.0 - fTemp48);
			self.fRec44[0] = -(fSlow10 * (fSlow11 * self.fRec44[1] - fSlow9 * (self.fRec7[1] - self.fRec7[2])));
			self.fRec45[0] = -(fSlow10 * (fSlow11 * self.fRec45[1] - (self.fRec7[1] + self.fRec7[2])));
			let mut fTemp53: F32 = self.fRec44[0] + fSlow12 * self.fRec45[0];
			self.fVec14[0] = fTemp53;
			self.fRec43[0] = -(fSlow7 * (fSlow8 * self.fRec43[1] - (fTemp53 + self.fVec14[1])));
			self.fRec46[0] = -(fSlow7 * (fSlow8 * self.fRec46[1] - fSlow6 * (fTemp53 - self.fVec14[1])));
			let mut fTemp54: F32 = fSlow39 * (self.fRec43[0] + fSlow13 * self.fRec46[0]);
			self.fVec15[(self.IOTA0 & 131071) as usize] = fTemp54;
			let mut iTemp55: i32 = (fTemp47) as i32;
			let mut fTemp56: F32 = 0.16666667 * fTemp51 * fTemp52 * self.fVec15[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp55, 3))) as F32)) as i32)) & 131071) as usize] + fTemp31 + (fTemp46 + (-3.0 - fTemp48)) * (fTemp52 * (0.5 * fTemp49 * self.fVec15[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp55, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec15[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp55)) as F32)) as i32)) & 131071) as usize] * fTemp50) - 0.5 * fTemp51 * self.fVec15[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp55, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec40[1];
			self.fVec16[(self.IOTA0 & 1023) as usize] = fTemp56;
			self.fRec40[0] = self.fVec16[((i32::wrapping_sub(self.IOTA0, 852)) & 1023) as usize];
			let mut fRec41: F32 = 0.6 * fTemp56;
			let mut fTemp57: F32 = F32::max(1.0, fSlow1 * (fTemp45 + 0.25688207));
			let mut fTemp58: F32 = fTemp57 + -0.999995;
			let mut fTemp59: F32 = F32::floor(fTemp58);
			let mut fTemp60: F32 = fTemp57 - fTemp59;
			let mut fTemp61: F32 = fTemp57 + (-1.0 - fTemp59);
			let mut fTemp62: F32 = fTemp60 * fTemp61;
			let mut fTemp63: F32 = fTemp57 + (-2.0 - fTemp59);
			self.fRec50[0] = -(fSlow10 * (fSlow11 * self.fRec50[1] - fSlow9 * (self.fRec3[1] - self.fRec3[2])));
			self.fRec51[0] = -(fSlow10 * (fSlow11 * self.fRec51[1] - (self.fRec3[1] + self.fRec3[2])));
			let mut fTemp64: F32 = self.fRec50[0] + fSlow12 * self.fRec51[0];
			self.fVec17[0] = fTemp64;
			self.fRec49[0] = -(fSlow7 * (fSlow8 * self.fRec49[1] - (fTemp64 + self.fVec17[1])));
			self.fRec52[0] = -(fSlow7 * (fSlow8 * self.fRec52[1] - fSlow6 * (fTemp64 - self.fVec17[1])));
			let mut fTemp65: F32 = fSlow40 * (self.fRec49[0] + fSlow13 * self.fRec52[0]);
			self.fVec18[(self.IOTA0 & 131071) as usize] = fTemp65;
			let mut iTemp66: i32 = (fTemp58) as i32;
			let mut fTemp67: F32 = 0.16666667 * fTemp62 * fTemp63 * self.fVec18[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp66, 3))) as F32)) as i32)) & 131071) as usize] + fTemp31 + (fTemp57 + (-3.0 - fTemp59)) * (fTemp63 * (0.5 * fTemp60 * self.fVec18[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp66, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec18[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp66)) as F32)) as i32)) & 131071) as usize] * fTemp61) - 0.5 * fTemp62 * self.fVec18[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp66, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec47[1];
			self.fVec19[(self.IOTA0 & 1023) as usize] = fTemp67;
			self.fRec47[0] = self.fVec19[((i32::wrapping_sub(self.IOTA0, 875)) & 1023) as usize];
			let mut fRec48: F32 = 0.6 * fTemp67;
			let mut fTemp68: F32 = (if iTemp0 != 0 {0.125} else {fSlow3 + self.fRec55[1]});
			self.fRec55[0] = fTemp68 - F32::floor(fTemp68);
			let mut fTemp69: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec55[0]);
			let mut fTemp70: F32 = F32::max(1.0, fSlow1 * (0.11940046 - fTemp69));
			let mut fTemp71: F32 = fTemp70 + -0.999995;
			let mut fTemp72: F32 = F32::floor(fTemp71);
			let mut fTemp73: F32 = fTemp70 - fTemp72;
			let mut fTemp74: F32 = fTemp70 + (-1.0 - fTemp72);
			let mut fTemp75: F32 = fTemp73 * fTemp74;
			let mut fTemp76: F32 = fTemp70 + (-2.0 - fTemp72);
			self.fRec57[0] = -(fSlow10 * (fSlow11 * self.fRec57[1] - fSlow9 * (self.fRec14[1] - self.fRec14[2])));
			self.fRec58[0] = -(fSlow10 * (fSlow11 * self.fRec58[1] - (self.fRec14[1] + self.fRec14[2])));
			let mut fTemp77: F32 = self.fRec57[0] + fSlow12 * self.fRec58[0];
			self.fVec20[0] = fTemp77;
			self.fRec56[0] = -(fSlow7 * (fSlow8 * self.fRec56[1] - (fTemp77 + self.fVec20[1])));
			self.fRec59[0] = -(fSlow7 * (fSlow8 * self.fRec59[1] - fSlow6 * (fTemp77 - self.fVec20[1])));
			let mut fTemp78: F32 = fSlow41 * (self.fRec56[0] + fSlow13 * self.fRec59[0]);
			self.fVec21[(self.IOTA0 & 131071) as usize] = fTemp78;
			let mut iTemp79: i32 = (fTemp71) as i32;
			let mut fTemp80: F32 = 0.16666667 * fTemp75 * fTemp76 * self.fVec21[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp79, 3))) as F32)) as i32)) & 131071) as usize] + fTemp15 + (fTemp70 + (-3.0 - fTemp72)) * (fTemp76 * (0.5 * fTemp73 * self.fVec21[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp79, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec21[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp79)) as F32)) as i32)) & 131071) as usize] * fTemp74) - 0.5 * fTemp75 * self.fVec21[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp79, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec53[1];
			self.fVec22[(self.IOTA0 & 1023) as usize] = fTemp80;
			self.fRec53[0] = self.fVec22[((i32::wrapping_sub(self.IOTA0, 662)) & 1023) as usize];
			let mut fRec54: F32 = 0.6 * fTemp80;
			let mut fTemp81: F32 = F32::max(1.0, fSlow1 * (fTemp69 + 0.08223061));
			let mut fTemp82: F32 = fTemp81 + -0.999995;
			let mut fTemp83: F32 = F32::floor(fTemp82);
			let mut fTemp84: F32 = fTemp81 - fTemp83;
			let mut fTemp85: F32 = fTemp81 + (-1.0 - fTemp83);
			let mut fTemp86: F32 = fTemp84 * fTemp85;
			let mut fTemp87: F32 = fTemp81 + (-2.0 - fTemp83);
			self.fRec63[0] = -(fSlow10 * (fSlow11 * self.fRec63[1] - fSlow9 * (self.fRec10[1] - self.fRec10[2])));
			self.fRec64[0] = -(fSlow10 * (fSlow11 * self.fRec64[1] - (self.fRec10[1] + self.fRec10[2])));
			let mut fTemp88: F32 = self.fRec63[0] + fSlow12 * self.fRec64[0];
			self.fVec23[0] = fTemp88;
			self.fRec62[0] = -(fSlow7 * (fSlow8 * self.fRec62[1] - (fTemp88 + self.fVec23[1])));
			self.fRec65[0] = -(fSlow7 * (fSlow8 * self.fRec65[1] - fSlow6 * (fTemp88 - self.fVec23[1])));
			let mut fTemp89: F32 = fSlow42 * (self.fRec62[0] + fSlow13 * self.fRec65[0]);
			self.fVec24[(self.IOTA0 & 65535) as usize] = fTemp89;
			let mut iTemp90: i32 = (fTemp82) as i32;
			let mut fTemp91: F32 = 0.16666667 * fTemp86 * fTemp87 * self.fVec24[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp90, 3))) as F32)) as i32)) & 65535) as usize] + fTemp15 + (fTemp81 + (-3.0 - fTemp83)) * (fTemp87 * (0.5 * fTemp84 * self.fVec24[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp90, 1))) as F32)) as i32)) & 65535) as usize] - 0.16666667 * self.fVec24[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp90)) as F32)) as i32)) & 65535) as usize] * fTemp85) - 0.5 * fTemp86 * self.fVec24[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp90, 2))) as F32)) as i32)) & 65535) as usize]) - 0.6 * self.fRec60[1];
			self.fVec25[(self.IOTA0 & 1023) as usize] = fTemp91;
			self.fRec60[0] = self.fVec25[((i32::wrapping_sub(self.IOTA0, 710)) & 1023) as usize];
			let mut fRec61: F32 = 0.6 * fTemp91;
			let mut fTemp92: F32 = (if iTemp0 != 0 {0.375} else {fSlow3 + self.fRec68[1]});
			self.fRec68[0] = fTemp92 - F32::floor(fTemp92);
			let mut fTemp93: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec68[0]);
			let mut fTemp94: F32 = F32::max(1.0, fSlow1 * (0.19230045 - fTemp93));
			let mut fTemp95: F32 = fTemp94 + -0.999995;
			let mut fTemp96: F32 = F32::floor(fTemp95);
			let mut fTemp97: F32 = fTemp94 - fTemp96;
			let mut fTemp98: F32 = fTemp94 + (-1.0 - fTemp96);
			let mut fTemp99: F32 = fTemp97 * fTemp98;
			let mut fTemp100: F32 = fTemp94 + (-2.0 - fTemp96);
			self.fRec70[0] = -(fSlow10 * (fSlow11 * self.fRec70[1] - fSlow9 * (self.fRec6[1] - self.fRec6[2])));
			self.fRec71[0] = -(fSlow10 * (fSlow11 * self.fRec71[1] - (self.fRec6[1] + self.fRec6[2])));
			let mut fTemp101: F32 = self.fRec70[0] + fSlow12 * self.fRec71[0];
			self.fVec26[0] = fTemp101;
			self.fRec69[0] = -(fSlow7 * (fSlow8 * self.fRec69[1] - (fTemp101 + self.fVec26[1])));
			self.fRec72[0] = -(fSlow7 * (fSlow8 * self.fRec72[1] - fSlow6 * (fTemp101 - self.fVec26[1])));
			let mut fTemp102: F32 = fSlow43 * (self.fRec69[0] + fSlow13 * self.fRec72[0]);
			self.fVec27[(self.IOTA0 & 131071) as usize] = fTemp102;
			let mut iTemp103: i32 = (fTemp95) as i32;
			let mut fTemp104: F32 = 0.16666667 * fTemp99 * fTemp100 * self.fVec27[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp103, 3))) as F32)) as i32)) & 131071) as usize] + fTemp15 + (fTemp94 + (-3.0 - fTemp96)) * (fTemp100 * (0.5 * fTemp97 * self.fVec27[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp103, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec27[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp103)) as F32)) as i32)) & 131071) as usize] * fTemp98) - 0.5 * fTemp99 * self.fVec27[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp103, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec66[1];
			self.fVec28[(self.IOTA0 & 1023) as usize] = fTemp104;
			self.fRec66[0] = self.fVec28[((i32::wrapping_sub(self.IOTA0, 906)) & 1023) as usize];
			let mut fRec67: F32 = 0.6 * fTemp104;
			let mut fTemp105: F32 = F32::max(1.0, fSlow1 * (fTemp93 + 0.17470522));
			let mut fTemp106: F32 = fTemp105 + -0.999995;
			let mut fTemp107: F32 = F32::floor(fTemp106);
			let mut fTemp108: F32 = fTemp105 - fTemp107;
			let mut fTemp109: F32 = fTemp105 + (-1.0 - fTemp107);
			let mut fTemp110: F32 = fTemp108 * fTemp109;
			let mut fTemp111: F32 = fTemp105 + (-2.0 - fTemp107);
			self.fRec76[0] = -(fSlow10 * (fSlow11 * self.fRec76[1] - fSlow9 * (self.fRec2[1] - self.fRec2[2])));
			self.fRec77[0] = -(fSlow10 * (fSlow11 * self.fRec77[1] - (self.fRec2[1] + self.fRec2[2])));
			let mut fTemp112: F32 = self.fRec76[0] + fSlow12 * self.fRec77[0];
			self.fVec29[0] = fTemp112;
			self.fRec75[0] = -(fSlow7 * (fSlow8 * self.fRec75[1] - (fTemp112 + self.fVec29[1])));
			self.fRec78[0] = -(fSlow7 * (fSlow8 * self.fRec78[1] - fSlow6 * (fTemp112 - self.fVec29[1])));
			let mut fTemp113: F32 = fSlow44 * (self.fRec75[0] + fSlow13 * self.fRec78[0]);
			self.fVec30[(self.IOTA0 & 131071) as usize] = fTemp113;
			let mut iTemp114: i32 = (fTemp106) as i32;
			let mut fTemp115: F32 = 0.16666667 * fTemp110 * fTemp111 * self.fVec30[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp114, 3))) as F32)) as i32)) & 131071) as usize] + fTemp15 + (fTemp105 + (-3.0 - fTemp107)) * (fTemp111 * (0.5 * fTemp108 * self.fVec30[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp114, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec30[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp114)) as F32)) as i32)) & 131071) as usize] * fTemp109) - 0.5 * fTemp110 * self.fVec30[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp114, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec73[1];
			self.fVec31[(self.IOTA0 & 1023) as usize] = fTemp115;
			self.fRec73[0] = self.fVec31[((i32::wrapping_sub(self.IOTA0, 932)) & 1023) as usize];
			let mut fRec74: F32 = 0.6 * fTemp115;
			let mut fTemp116: F32 = (if iTemp0 != 0 {0.0625} else {fSlow3 + self.fRec81[1]});
			self.fRec81[0] = fTemp116 - F32::floor(fTemp116);
			let mut fTemp117: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec81[0]);
			let mut fTemp118: F32 = F32::max(1.0, fSlow1 * (0.14782245 - fTemp117));
			let mut fTemp119: F32 = fTemp118 + -0.999995;
			let mut fTemp120: F32 = F32::floor(fTemp119);
			let mut fTemp121: F32 = fTemp118 - fTemp120;
			let mut fTemp122: F32 = fTemp118 + (-1.0 - fTemp120);
			let mut fTemp123: F32 = fTemp121 * fTemp122;
			let mut fTemp124: F32 = fTemp118 + (-2.0 - fTemp120);
			self.fRec83[0] = -(fSlow10 * (fSlow11 * self.fRec83[1] - fSlow9 * (self.fRec13[1] - self.fRec13[2])));
			self.fRec84[0] = -(fSlow10 * (fSlow11 * self.fRec84[1] - (self.fRec13[1] + self.fRec13[2])));
			let mut fTemp125: F32 = self.fRec83[0] + fSlow12 * self.fRec84[0];
			self.fVec32[0] = fTemp125;
			self.fRec82[0] = -(fSlow7 * (fSlow8 * self.fRec82[1] - (fTemp125 + self.fVec32[1])));
			self.fRec85[0] = -(fSlow7 * (fSlow8 * self.fRec85[1] - fSlow6 * (fTemp125 - self.fVec32[1])));
			let mut fTemp126: F32 = fSlow45 * (self.fRec82[0] + fSlow13 * self.fRec85[0]);
			self.fVec33[(self.IOTA0 & 131071) as usize] = fTemp126;
			let mut iTemp127: i32 = (fTemp119) as i32;
			let mut fTemp128: F32 = 0.16666667 * fTemp123 * fTemp124 * self.fVec33[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp127, 3))) as F32)) as i32)) & 131071) as usize] + fTemp31 + (fTemp118 + (-3.0 - fTemp120)) * (fTemp124 * (0.5 * fTemp121 * self.fVec33[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp127, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec33[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp127)) as F32)) as i32)) & 131071) as usize] * fTemp122) - 0.5 * fTemp123 * self.fVec33[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp127, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec79[1];
			self.fVec34[(self.IOTA0 & 1023) as usize] = fTemp128;
			self.fRec79[0] = self.fVec34[((i32::wrapping_sub(self.IOTA0, 778)) & 1023) as usize];
			let mut fRec80: F32 = 0.6 * fTemp128;
			let mut fTemp129: F32 = F32::max(1.0, fSlow1 * (fTemp117 + 0.07776644));
			let mut fTemp130: F32 = fTemp129 + -0.999995;
			let mut fTemp131: F32 = F32::floor(fTemp130);
			let mut fTemp132: F32 = fTemp129 - fTemp131;
			let mut fTemp133: F32 = fTemp129 + (-1.0 - fTemp131);
			let mut fTemp134: F32 = fTemp132 * fTemp133;
			let mut fTemp135: F32 = fTemp129 + (-2.0 - fTemp131);
			self.fRec89[0] = -(fSlow10 * (fSlow11 * self.fRec89[1] - fSlow9 * (self.fRec9[1] - self.fRec9[2])));
			self.fRec90[0] = -(fSlow10 * (fSlow11 * self.fRec90[1] - (self.fRec9[1] + self.fRec9[2])));
			let mut fTemp136: F32 = self.fRec89[0] + fSlow12 * self.fRec90[0];
			self.fVec35[0] = fTemp136;
			self.fRec88[0] = -(fSlow7 * (fSlow8 * self.fRec88[1] - (fTemp136 + self.fVec35[1])));
			self.fRec91[0] = -(fSlow7 * (fSlow8 * self.fRec91[1] - fSlow6 * (fTemp136 - self.fVec35[1])));
			let mut fTemp137: F32 = fSlow46 * (self.fRec88[0] + fSlow13 * self.fRec91[0]);
			self.fVec36[(self.IOTA0 & 65535) as usize] = fTemp137;
			let mut iTemp138: i32 = (fTemp130) as i32;
			let mut fTemp139: F32 = 0.16666667 * fTemp134 * fTemp135 * self.fVec36[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp138, 3))) as F32)) as i32)) & 65535) as usize] + fTemp31 + (fTemp129 + (-3.0 - fTemp131)) * (fTemp135 * (0.5 * fTemp132 * self.fVec36[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp138, 1))) as F32)) as i32)) & 65535) as usize] - 0.16666667 * self.fVec36[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp138)) as F32)) as i32)) & 65535) as usize] * fTemp133) - 0.5 * fTemp134 * self.fVec36[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp138, 2))) as F32)) as i32)) & 65535) as usize]) - 0.6 * self.fRec86[1];
			self.fVec37[(self.IOTA0 & 1023) as usize] = fTemp139;
			self.fRec86[0] = self.fVec37[((i32::wrapping_sub(self.IOTA0, 1001)) & 1023) as usize];
			let mut fRec87: F32 = 0.6 * fTemp139;
			let mut fTemp140: F32 = (if iTemp0 != 0 {0.3125} else {fSlow3 + self.fRec94[1]});
			self.fRec94[0] = fTemp140 - F32::floor(fTemp140);
			let mut fTemp141: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec94[0]);
			let mut fTemp142: F32 = F32::max(1.0, fSlow1 * (0.125 - fTemp141));
			let mut fTemp143: F32 = fTemp142 + -0.999995;
			let mut fTemp144: F32 = F32::floor(fTemp143);
			let mut fTemp145: F32 = fTemp142 - fTemp144;
			let mut fTemp146: F32 = fTemp142 + (-1.0 - fTemp144);
			let mut fTemp147: F32 = fTemp145 * fTemp146;
			let mut fTemp148: F32 = fTemp142 + (-2.0 - fTemp144);
			self.fRec96[0] = -(fSlow10 * (fSlow11 * self.fRec96[1] - fSlow9 * (self.fRec5[1] - self.fRec5[2])));
			self.fRec97[0] = -(fSlow10 * (fSlow11 * self.fRec97[1] - (self.fRec5[1] + self.fRec5[2])));
			let mut fTemp149: F32 = self.fRec96[0] + fSlow12 * self.fRec97[0];
			self.fVec38[0] = fTemp149;
			self.fRec95[0] = -(fSlow7 * (fSlow8 * self.fRec95[1] - (fTemp149 + self.fVec38[1])));
			self.fRec98[0] = -(fSlow7 * (fSlow8 * self.fRec98[1] - fSlow6 * (fTemp149 - self.fVec38[1])));
			let mut fTemp150: F32 = fSlow47 * (self.fRec95[0] + fSlow13 * self.fRec98[0]);
			self.fVec39[(self.IOTA0 & 131071) as usize] = fTemp150;
			let mut iTemp151: i32 = (fTemp143) as i32;
			let mut fTemp152: F32 = 0.16666667 * fTemp147 * fTemp148 * self.fVec39[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp151, 3))) as F32)) as i32)) & 131071) as usize] + fTemp31 + (fTemp142 + (-3.0 - fTemp144)) * (fTemp148 * (0.5 * fTemp145 * self.fVec39[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp151, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec39[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp151)) as F32)) as i32)) & 131071) as usize] * fTemp146) - 0.5 * fTemp147 * self.fVec39[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp151, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec92[1];
			self.fVec40[(self.IOTA0 & 1023) as usize] = fTemp152;
			self.fRec92[0] = self.fVec40[((i32::wrapping_sub(self.IOTA0, 806)) & 1023) as usize];
			let mut fRec93: F32 = 0.6 * fTemp152;
			let mut fTemp153: F32 = F32::max(1.0, fSlow1 * (fTemp141 + 0.21039456));
			let mut fTemp154: F32 = fTemp153 + -0.999995;
			let mut fTemp155: F32 = F32::floor(fTemp154);
			let mut fTemp156: F32 = fTemp153 - fTemp155;
			let mut fTemp157: F32 = fTemp153 + (-1.0 - fTemp155);
			let mut fTemp158: F32 = fTemp156 * fTemp157;
			let mut fTemp159: F32 = fTemp153 + (-2.0 - fTemp155);
			self.fRec102[0] = -(fSlow10 * (fSlow11 * self.fRec102[1] - fSlow9 * (self.fRec1[1] - self.fRec1[2])));
			self.fRec103[0] = -(fSlow10 * (fSlow11 * self.fRec103[1] - (self.fRec1[1] + self.fRec1[2])));
			let mut fTemp160: F32 = self.fRec102[0] + fSlow12 * self.fRec103[0];
			self.fVec41[0] = fTemp160;
			self.fRec101[0] = -(fSlow7 * (fSlow8 * self.fRec101[1] - (fTemp160 + self.fVec41[1])));
			self.fRec104[0] = -(fSlow7 * (fSlow8 * self.fRec104[1] - fSlow6 * (fTemp160 - self.fVec41[1])));
			let mut fTemp161: F32 = fSlow48 * (self.fRec101[0] + fSlow13 * self.fRec104[0]);
			self.fVec42[(self.IOTA0 & 131071) as usize] = fTemp161;
			let mut iTemp162: i32 = (fTemp154) as i32;
			let mut fTemp163: F32 = 0.16666667 * fTemp158 * fTemp159 * self.fVec42[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp162, 3))) as F32)) as i32)) & 131071) as usize] + (fTemp153 + (-3.0 - fTemp155)) * (fTemp159 * (0.5 * fTemp156 * self.fVec42[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp162, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec42[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp162)) as F32)) as i32)) & 131071) as usize] * fTemp157) - 0.5 * fTemp158 * self.fVec42[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp162, 2))) as F32)) as i32)) & 131071) as usize]) + fTemp31 - 0.6 * self.fRec99[1];
			self.fVec43[(self.IOTA0 & 1023) as usize] = fTemp163;
			self.fRec99[0] = self.fVec43[((i32::wrapping_sub(self.IOTA0, 798)) & 1023) as usize];
			let mut fRec100: F32 = 0.6 * fTemp163;
			let mut fTemp164: F32 = (if iTemp0 != 0 {0.0} else {fSlow3 + self.fRec107[1]});
			self.fRec107[0] = fTemp164 - F32::floor(fTemp164);
			let mut fTemp165: F32 = fSlow2 * F32::sin(6.2831855 * self.fRec107[0]);
			let mut fTemp166: F32 = F32::max(1.0, fSlow1 * (0.10252925 - fTemp165));
			let mut fTemp167: F32 = fTemp166 + -0.999995;
			let mut fTemp168: F32 = F32::floor(fTemp167);
			let mut fTemp169: F32 = fTemp166 - fTemp168;
			let mut fTemp170: F32 = fTemp166 + (-1.0 - fTemp168);
			let mut fTemp171: F32 = fTemp169 * fTemp170;
			let mut fTemp172: F32 = fTemp166 + (-2.0 - fTemp168);
			self.fRec109[0] = -(fSlow10 * (fSlow11 * self.fRec109[1] - fSlow9 * (self.fRec12[1] - self.fRec12[2])));
			self.fRec110[0] = -(fSlow10 * (fSlow11 * self.fRec110[1] - (self.fRec12[1] + self.fRec12[2])));
			let mut fTemp173: F32 = self.fRec109[0] + fSlow12 * self.fRec110[0];
			self.fVec44[0] = fTemp173;
			self.fRec108[0] = -(fSlow7 * (fSlow8 * self.fRec108[1] - (fTemp173 + self.fVec44[1])));
			self.fRec111[0] = -(fSlow7 * (fSlow8 * self.fRec111[1] - fSlow6 * (fTemp173 - self.fVec44[1])));
			let mut fTemp174: F32 = fSlow49 * (self.fRec108[0] + fSlow13 * self.fRec111[0]);
			self.fVec45[(self.IOTA0 & 65535) as usize] = fTemp174;
			let mut iTemp175: i32 = (fTemp167) as i32;
			let mut fTemp176: F32 = 0.16666667 * fTemp171 * fTemp172 * self.fVec45[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp175, 3))) as F32)) as i32)) & 65535) as usize] + fTemp15 + (fTemp166 + (-3.0 - fTemp168)) * (fTemp172 * (0.5 * fTemp169 * self.fVec45[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp175, 1))) as F32)) as i32)) & 65535) as usize] - 0.16666667 * self.fVec45[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp175)) as F32)) as i32)) & 65535) as usize] * fTemp170) - 0.5 * fTemp171 * self.fVec45[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp175, 2))) as F32)) as i32)) & 65535) as usize]) - 0.6 * self.fRec105[1];
			self.fVec46[(self.IOTA0 & 1023) as usize] = fTemp176;
			self.fRec105[0] = self.fVec46[((i32::wrapping_sub(self.IOTA0, 832)) & 1023) as usize];
			let mut fRec106: F32 = 0.6 * fTemp176;
			let mut fTemp177: F32 = F32::max(1.0, fSlow1 * (fTemp165 + 0.070764855));
			let mut fTemp178: F32 = fTemp177 + -0.999995;
			let mut fTemp179: F32 = F32::floor(fTemp178);
			let mut fTemp180: F32 = fTemp177 - fTemp179;
			let mut fTemp181: F32 = fTemp177 + (-1.0 - fTemp179);
			let mut fTemp182: F32 = fTemp180 * fTemp181;
			let mut fTemp183: F32 = fTemp177 + (-2.0 - fTemp179);
			self.fRec115[0] = -(fSlow10 * (fSlow11 * self.fRec115[1] - fSlow9 * (self.fRec8[1] - self.fRec8[2])));
			self.fRec116[0] = -(fSlow10 * (fSlow11 * self.fRec116[1] - (self.fRec8[1] + self.fRec8[2])));
			let mut fTemp184: F32 = self.fRec115[0] + fSlow12 * self.fRec116[0];
			self.fVec47[0] = fTemp184;
			self.fRec114[0] = -(fSlow7 * (fSlow8 * self.fRec114[1] - (fTemp184 + self.fVec47[1])));
			self.fRec117[0] = -(fSlow7 * (fSlow8 * self.fRec117[1] - fSlow6 * (fTemp184 - self.fVec47[1])));
			let mut fTemp185: F32 = fSlow50 * (self.fRec114[0] + fSlow13 * self.fRec117[0]);
			self.fVec48[(self.IOTA0 & 65535) as usize] = fTemp185;
			let mut iTemp186: i32 = (fTemp178) as i32;
			let mut fTemp187: F32 = 0.16666667 * fTemp182 * fTemp183 * self.fVec48[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp186, 3))) as F32)) as i32)) & 65535) as usize] + fTemp15 + (fTemp177 + (-3.0 - fTemp179)) * (fTemp183 * (0.5 * fTemp180 * self.fVec48[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp186, 1))) as F32)) as i32)) & 65535) as usize] - 0.16666667 * self.fVec48[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp186)) as F32)) as i32)) & 65535) as usize] * fTemp181) - 0.5 * fTemp182 * self.fVec48[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp186, 2))) as F32)) as i32)) & 65535) as usize]) - 0.6 * self.fRec112[1];
			self.fVec49[(self.IOTA0 & 1023) as usize] = fTemp187;
			self.fRec112[0] = self.fVec49[((i32::wrapping_sub(self.IOTA0, 956)) & 1023) as usize];
			let mut fRec113: F32 = 0.6 * fTemp187;
			let mut fTemp188: F32 = F32::max(1.0, fSlow1 * (0.21998005 - fTemp2));
			let mut fTemp189: F32 = fTemp188 + -0.999995;
			let mut fTemp190: F32 = F32::floor(fTemp189);
			let mut fTemp191: F32 = fTemp188 - fTemp190;
			let mut fTemp192: F32 = fTemp188 + (-1.0 - fTemp190);
			let mut fTemp193: F32 = fTemp191 * fTemp192;
			let mut fTemp194: F32 = fTemp188 + (-2.0 - fTemp190);
			self.fRec121[0] = -(fSlow10 * (fSlow11 * self.fRec121[1] - fSlow9 * (self.fRec4[1] - self.fRec4[2])));
			self.fRec122[0] = -(fSlow10 * (fSlow11 * self.fRec122[1] - (self.fRec4[1] + self.fRec4[2])));
			let mut fTemp195: F32 = self.fRec121[0] + fSlow12 * self.fRec122[0];
			self.fVec50[0] = fTemp195;
			self.fRec120[0] = -(fSlow7 * (fSlow8 * self.fRec120[1] - (fTemp195 + self.fVec50[1])));
			self.fRec123[0] = -(fSlow7 * (fSlow8 * self.fRec123[1] - fSlow6 * (fTemp195 - self.fVec50[1])));
			let mut fTemp196: F32 = fSlow51 * (self.fRec120[0] + fSlow13 * self.fRec123[0]);
			self.fVec51[(self.IOTA0 & 131071) as usize] = fTemp196;
			let mut iTemp197: i32 = (fTemp189) as i32;
			let mut fTemp198: F32 = 0.16666667 * fTemp193 * fTemp194 * self.fVec51[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp197, 3))) as F32)) as i32)) & 131071) as usize] + fTemp15 + (fTemp188 + (-3.0 - fTemp190)) * (fTemp194 * (0.5 * fTemp191 * self.fVec51[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp197, 1))) as F32)) as i32)) & 131071) as usize] - 0.16666667 * self.fVec51[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, iTemp197)) as F32)) as i32)) & 131071) as usize] * fTemp192) - 0.5 * fTemp193 * self.fVec51[((i32::wrapping_sub(self.IOTA0, (F32::min(self.fConst3, (std::cmp::max(0, i32::wrapping_add(iTemp197, 2))) as F32)) as i32)) & 131071) as usize]) - 0.6 * self.fRec118[1];
			self.fVec52[(self.IOTA0 & 1023) as usize] = fTemp198;
			self.fRec118[0] = self.fVec52[((i32::wrapping_sub(self.IOTA0, 894)) & 1023) as usize];
			let mut fRec119: F32 = 0.6 * fTemp198;
			let mut fTemp199: F32 = fRec106 + fRec113 + fRec17 + fRec119;
			let mut fTemp200: F32 = 0.25 * (self.fRec25[1] + self.fRec34[1] + self.fRec40[1] + self.fRec47[1] + self.fRec53[1] + self.fRec60[1] + self.fRec66[1] + self.fRec73[1] + self.fRec79[1] + self.fRec86[1] + self.fRec92[1] + self.fRec99[1] + self.fRec105[1] + self.fRec112[1] + self.fRec118[1] + self.fRec16[1] + fRec26 + fRec35 + fRec41 + fRec48 + fRec54 + fRec61 + fRec67 + fRec74 + fRec80 + fRec87 + fRec93 + fRec100 + fTemp199);
			let mut fTemp201: F32 = self.fRec105[1] + self.fRec112[1] + self.fRec118[1] + self.fRec16[1] + fTemp199;
			let mut fTemp202: F32 = self.fRec47[1] + self.fRec73[1] + self.fRec99[1] + self.fRec16[1] + fRec48 + fRec74 + fRec17 + fRec100;
			self.fRec0[0] = fRec17 + self.fRec16[1] + fTemp200 - 0.5 * (fTemp201 + fTemp202);
			let mut fTemp203: F32 = self.fRec79[1] + self.fRec86[1] + self.fRec92[1] + self.fRec99[1] + fRec80 + fRec87 + fRec100 + fRec93;
			self.fRec1[0] = fRec100 + self.fRec99[1] + fTemp200 - 0.5 * (fTemp203 + fTemp202);
			let mut fTemp204: F32 = self.fRec53[1] + self.fRec60[1] + self.fRec66[1] + self.fRec73[1] + fRec54 + fRec61 + fRec74 + fRec67;
			self.fRec2[0] = fRec74 + self.fRec73[1] + fTemp200 - 0.5 * (fTemp204 + fTemp202);
			let mut fTemp205: F32 = self.fRec25[1] + self.fRec34[1] + self.fRec40[1] + self.fRec47[1] + fRec26 + fRec35 + fRec48 + fRec41;
			self.fRec3[0] = fRec48 + self.fRec47[1] + fTemp200 - 0.5 * (fTemp205 + fTemp202);
			let mut fTemp206: F32 = self.fRec40[1] + self.fRec66[1] + self.fRec92[1] + self.fRec118[1] + fRec41 + fRec67 + fRec119 + fRec93;
			self.fRec4[0] = fRec119 + self.fRec118[1] + fTemp200 - 0.5 * (fTemp201 + fTemp206);
			self.fRec5[0] = fRec93 + self.fRec92[1] + fTemp200 - 0.5 * (fTemp203 + fTemp206);
			self.fRec6[0] = fRec67 + self.fRec66[1] + fTemp200 - 0.5 * (fTemp204 + fTemp206);
			self.fRec7[0] = fRec41 + self.fRec40[1] + fTemp200 - 0.5 * (fTemp205 + fTemp206);
			let mut fTemp207: F32 = self.fRec34[1] + self.fRec60[1] + self.fRec86[1] + self.fRec112[1] + fRec35 + fRec61 + fRec113 + fRec87;
			self.fRec8[0] = fRec113 + self.fRec112[1] + fTemp200 - 0.5 * (fTemp201 + fTemp207);
			self.fRec9[0] = fRec87 + self.fRec86[1] + fTemp200 - 0.5 * (fTemp203 + fTemp207);
			self.fRec10[0] = fRec61 + self.fRec60[1] + fTemp200 - 0.5 * (fTemp204 + fTemp207);
			self.fRec11[0] = fRec35 + self.fRec34[1] + fTemp200 - 0.5 * (fTemp205 + fTemp207);
			let mut fTemp208: F32 = self.fRec25[1] + self.fRec53[1] + self.fRec79[1] + self.fRec105[1] + fRec26 + fRec54 + fRec106 + fRec80;
			self.fRec12[0] = fRec106 + self.fRec105[1] + fTemp200 - 0.5 * (fTemp201 + fTemp208);
			self.fRec13[0] = fRec80 + self.fRec79[1] + fTemp200 - 0.5 * (fTemp203 + fTemp208);
			self.fRec14[0] = fRec54 + self.fRec53[1] + fTemp200 - 0.5 * (fTemp204 + fTemp208);
			self.fRec15[0] = fRec26 + self.fRec25[1] + fTemp200 - 0.5 * (fTemp205 + fTemp208);
			*output0 = 0.5 * (self.fRec1[0] + self.fRec3[0] + self.fRec5[0] + self.fRec7[0] + self.fRec9[0] + self.fRec11[0] + self.fRec13[0] + self.fRec15[0]);
			*output1 = 0.5 * (self.fRec0[0] + self.fRec2[0] + self.fRec4[0] + self.fRec6[0] + self.fRec8[0] + self.fRec10[0] + self.fRec12[0] + self.fRec14[0]);
			self.iVec0[1] = self.iVec0[0];
			self.fRec18[1] = self.fRec18[0];
			self.fRec20[1] = self.fRec20[0];
			self.fRec21[1] = self.fRec21[0];
			self.fVec1[1] = self.fVec1[0];
			self.fRec19[1] = self.fRec19[0];
			self.fRec22[1] = self.fRec22[0];
			self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
			self.fVec4[1] = self.fVec4[0];
			self.fRec24[1] = self.fRec24[0];
			self.fRec23[1] = self.fRec23[0];
			self.fRec16[1] = self.fRec16[0];
			self.fRec27[1] = self.fRec27[0];
			self.fRec29[1] = self.fRec29[0];
			self.fRec30[1] = self.fRec30[0];
			self.fVec6[1] = self.fVec6[0];
			self.fRec28[1] = self.fRec28[0];
			self.fRec31[1] = self.fRec31[0];
			self.fVec9[1] = self.fVec9[0];
			self.fRec33[1] = self.fRec33[0];
			self.fRec32[1] = self.fRec32[0];
			self.fRec25[1] = self.fRec25[0];
			self.fRec37[1] = self.fRec37[0];
			self.fRec38[1] = self.fRec38[0];
			self.fVec11[1] = self.fVec11[0];
			self.fRec36[1] = self.fRec36[0];
			self.fRec39[1] = self.fRec39[0];
			self.fRec34[1] = self.fRec34[0];
			self.fRec42[1] = self.fRec42[0];
			self.fRec44[1] = self.fRec44[0];
			self.fRec45[1] = self.fRec45[0];
			self.fVec14[1] = self.fVec14[0];
			self.fRec43[1] = self.fRec43[0];
			self.fRec46[1] = self.fRec46[0];
			self.fRec40[1] = self.fRec40[0];
			self.fRec50[1] = self.fRec50[0];
			self.fRec51[1] = self.fRec51[0];
			self.fVec17[1] = self.fVec17[0];
			self.fRec49[1] = self.fRec49[0];
			self.fRec52[1] = self.fRec52[0];
			self.fRec47[1] = self.fRec47[0];
			self.fRec55[1] = self.fRec55[0];
			self.fRec57[1] = self.fRec57[0];
			self.fRec58[1] = self.fRec58[0];
			self.fVec20[1] = self.fVec20[0];
			self.fRec56[1] = self.fRec56[0];
			self.fRec59[1] = self.fRec59[0];
			self.fRec53[1] = self.fRec53[0];
			self.fRec63[1] = self.fRec63[0];
			self.fRec64[1] = self.fRec64[0];
			self.fVec23[1] = self.fVec23[0];
			self.fRec62[1] = self.fRec62[0];
			self.fRec65[1] = self.fRec65[0];
			self.fRec60[1] = self.fRec60[0];
			self.fRec68[1] = self.fRec68[0];
			self.fRec70[1] = self.fRec70[0];
			self.fRec71[1] = self.fRec71[0];
			self.fVec26[1] = self.fVec26[0];
			self.fRec69[1] = self.fRec69[0];
			self.fRec72[1] = self.fRec72[0];
			self.fRec66[1] = self.fRec66[0];
			self.fRec76[1] = self.fRec76[0];
			self.fRec77[1] = self.fRec77[0];
			self.fVec29[1] = self.fVec29[0];
			self.fRec75[1] = self.fRec75[0];
			self.fRec78[1] = self.fRec78[0];
			self.fRec73[1] = self.fRec73[0];
			self.fRec81[1] = self.fRec81[0];
			self.fRec83[1] = self.fRec83[0];
			self.fRec84[1] = self.fRec84[0];
			self.fVec32[1] = self.fVec32[0];
			self.fRec82[1] = self.fRec82[0];
			self.fRec85[1] = self.fRec85[0];
			self.fRec79[1] = self.fRec79[0];
			self.fRec89[1] = self.fRec89[0];
			self.fRec90[1] = self.fRec90[0];
			self.fVec35[1] = self.fVec35[0];
			self.fRec88[1] = self.fRec88[0];
			self.fRec91[1] = self.fRec91[0];
			self.fRec86[1] = self.fRec86[0];
			self.fRec94[1] = self.fRec94[0];
			self.fRec96[1] = self.fRec96[0];
			self.fRec97[1] = self.fRec97[0];
			self.fVec38[1] = self.fVec38[0];
			self.fRec95[1] = self.fRec95[0];
			self.fRec98[1] = self.fRec98[0];
			self.fRec92[1] = self.fRec92[0];
			self.fRec102[1] = self.fRec102[0];
			self.fRec103[1] = self.fRec103[0];
			self.fVec41[1] = self.fVec41[0];
			self.fRec101[1] = self.fRec101[0];
			self.fRec104[1] = self.fRec104[0];
			self.fRec99[1] = self.fRec99[0];
			self.fRec107[1] = self.fRec107[0];
			self.fRec109[1] = self.fRec109[0];
			self.fRec110[1] = self.fRec110[0];
			self.fVec44[1] = self.fVec44[0];
			self.fRec108[1] = self.fRec108[0];
			self.fRec111[1] = self.fRec111[0];
			self.fRec105[1] = self.fRec105[0];
			self.fRec115[1] = self.fRec115[0];
			self.fRec116[1] = self.fRec116[0];
			self.fVec47[1] = self.fVec47[0];
			self.fRec114[1] = self.fRec114[0];
			self.fRec117[1] = self.fRec117[0];
			self.fRec112[1] = self.fRec112[0];
			self.fRec121[1] = self.fRec121[0];
			self.fRec122[1] = self.fRec122[0];
			self.fVec50[1] = self.fVec50[0];
			self.fRec120[1] = self.fRec120[0];
			self.fRec123[1] = self.fRec123[0];
			self.fRec118[1] = self.fRec118[0];
			self.fRec0[2] = self.fRec0[1];
			self.fRec0[1] = self.fRec0[0];
			self.fRec1[2] = self.fRec1[1];
			self.fRec1[1] = self.fRec1[0];
			self.fRec2[2] = self.fRec2[1];
			self.fRec2[1] = self.fRec2[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fRec4[2] = self.fRec4[1];
			self.fRec4[1] = self.fRec4[0];
			self.fRec5[2] = self.fRec5[1];
			self.fRec5[1] = self.fRec5[0];
			self.fRec6[2] = self.fRec6[1];
			self.fRec6[1] = self.fRec6[0];
			self.fRec7[2] = self.fRec7[1];
			self.fRec7[1] = self.fRec7[0];
			self.fRec8[2] = self.fRec8[1];
			self.fRec8[1] = self.fRec8[0];
			self.fRec9[2] = self.fRec9[1];
			self.fRec9[1] = self.fRec9[0];
			self.fRec10[2] = self.fRec10[1];
			self.fRec10[1] = self.fRec10[0];
			self.fRec11[2] = self.fRec11[1];
			self.fRec11[1] = self.fRec11[0];
			self.fRec12[2] = self.fRec12[1];
			self.fRec12[1] = self.fRec12[0];
			self.fRec13[2] = self.fRec13[1];
			self.fRec13[1] = self.fRec13[0];
			self.fRec14[2] = self.fRec14[1];
			self.fRec14[1] = self.fRec14[0];
			self.fRec15[2] = self.fRec15[1];
			self.fRec15[1] = self.fRec15[0];
		}
		
	}

}

impl FaustDsp for VitalRevDsp {
	type T = FaustFloat;
	fn new() -> Self where Self: Sized {
		Self::new()
	}
	fn metadata(&self, m: &mut dyn Meta) {
		self.metadata(m)
	}
	fn get_sample_rate(&self) -> i32 {
		self.get_sample_rate()
	}
	fn get_num_inputs(&self) -> i32 {
		FAUST_INPUTS as i32
	}
	fn get_num_outputs(&self) -> i32 {
		FAUST_OUTPUTS as i32
	}
	fn class_init(sample_rate: i32) where Self: Sized {
		Self::class_init(sample_rate);
	}
	fn instance_reset_params(&mut self) {
		self.instance_reset_params()
	}
	fn instance_clear(&mut self) {
		self.instance_clear()
	}
	fn instance_constants(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate)
	}
	fn instance_init(&mut self, sample_rate: i32) {
		self.instance_init(sample_rate)
	}
	fn init(&mut self, sample_rate: i32) {
		self.init(sample_rate)
	}
	fn build_user_interface(&self, ui_interface: &mut dyn UI<Self::T>) {
		self.build_user_interface(ui_interface)
	}
	fn build_user_interface_static(ui_interface: &mut dyn UI<Self::T>) where Self: Sized {
		Self::build_user_interface_static(ui_interface);
	}
	fn get_param(&self, param: ParamIndex) -> Option<Self::T> {
		self.get_param(param)
	}
	fn set_param(&mut self, param: ParamIndex, value: Self::T) {
		self.set_param(param, value)
	}
	fn compute(&mut self, count: i32, inputs: &[&[Self::T]], outputs: &mut [&mut [Self::T]]) {
		self.compute(count as usize, inputs, outputs)
	}
}
