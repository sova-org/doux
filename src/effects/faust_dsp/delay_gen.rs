/* ------------------------------------------------------------
name: "delay"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn DelayDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct DelayDsp {
	fHslider0: F32,
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fConst2: F32,
	fHslider1: F32,
	fRec1: [F32;2],
	IOTA0: i32,
	fVec0: [F32;65536],
	fConst3: F32,
	fConst4: F32,
	fHslider2: F32,
	fRec2: [F32;2],
	fRec0: [F32;2],
	fVec1: [F32;65536],
	fRec3: [F32;2],
	fVec2: [F32;65536],
	fRec4: [F32;2],
	fRec6: [F32;2],
	fVec3: [F32;65536],
	fRec5: [F32;2],
	fVec4: [F32;65536],
	fRec7: [F32;2],
	fVec5: [F32;65536],
	fRec9: [F32;2],
	fRec11: [F32;2],
	fVec6: [F32;65536],
	fRec10: [F32;2],
	fVec7: [F32;65536],
	fRec12: [F32;2],
}

pub type FaustFloat = F32;
fn remainder_f32(from: f32, to: f32) -> f32 {
	from - to * (from / to).round_ties_even()
}
fn rint_f32(val: f32) -> f32 {
	val.round_ties_even()
}

pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 3;
pub const FAUST_PASSIVES: usize = 0;


impl DelayDsp {
		
	pub fn new() -> DelayDsp { 
		DelayDsp {
			fHslider0: 0.0,
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fConst2: 0.0,
			fHslider1: 0.0,
			fRec1: [0.0;2],
			IOTA0: 0,
			fVec0: [0.0;65536],
			fConst3: 0.0,
			fConst4: 0.0,
			fHslider2: 0.0,
			fRec2: [0.0;2],
			fRec0: [0.0;2],
			fVec1: [0.0;65536],
			fRec3: [0.0;2],
			fVec2: [0.0;65536],
			fRec4: [0.0;2],
			fRec6: [0.0;2],
			fVec3: [0.0;65536],
			fRec5: [0.0;2],
			fVec4: [0.0;65536],
			fRec7: [0.0;2],
			fVec5: [0.0;65536],
			fRec9: [0.0;2],
			fRec11: [0.0;2],
			fVec6: [0.0;65536],
			fRec10: [0.0;2],
			fVec7: [0.0;65536],
			fRec12: [0.0;2],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/version", r"1.21.0");
		m.declare("compile_options", r"-lang rust -ct 1 -cn DelayDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("delays.lib/name", r"Faust Delay Library");
		m.declare("delays.lib/version", r"1.2.0");
		m.declare("filename", r"delay.dsp");
		m.declare("filters.lib/lowpass0_highpass1", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/name", r"Faust Filters Library");
		m.declare("filters.lib/pole:author", r"Julius O. Smith III");
		m.declare("filters.lib/pole:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/pole:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/version", r"1.7.1");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"delay");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
		m.declare("signals.lib/name", r"Faust Signal Routing Library");
		m.declare("signals.lib/version", r"1.6.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.0;
		self.fHslider1 = 0.6;
		self.fHslider2 = 0.333;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fRec1[l0 as usize] = 0.0;
		}
		self.IOTA0 = 0;
		for l1 in 0..65536 {
			self.fVec0[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec2[l2 as usize] = 0.0;
		}
		for l3 in 0..2 {
			self.fRec0[l3 as usize] = 0.0;
		}
		for l4 in 0..65536 {
			self.fVec1[l4 as usize] = 0.0;
		}
		for l5 in 0..2 {
			self.fRec3[l5 as usize] = 0.0;
		}
		for l6 in 0..65536 {
			self.fVec2[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fRec4[l7 as usize] = 0.0;
		}
		for l8 in 0..2 {
			self.fRec6[l8 as usize] = 0.0;
		}
		for l9 in 0..65536 {
			self.fVec3[l9 as usize] = 0.0;
		}
		for l10 in 0..2 {
			self.fRec5[l10 as usize] = 0.0;
		}
		for l11 in 0..65536 {
			self.fVec4[l11 as usize] = 0.0;
		}
		for l12 in 0..2 {
			self.fRec7[l12 as usize] = 0.0;
		}
		for l13 in 0..65536 {
			self.fVec5[l13 as usize] = 0.0;
		}
		for l14 in 0..2 {
			self.fRec9[l14 as usize] = 0.0;
		}
		for l15 in 0..2 {
			self.fRec11[l15 as usize] = 0.0;
		}
		for l16 in 0..65536 {
			self.fVec6[l16 as usize] = 0.0;
		}
		for l17 in 0..2 {
			self.fRec10[l17 as usize] = 0.0;
		}
		for l18 in 0..65536 {
			self.fVec7[l18 as usize] = 0.0;
		}
		for l19 in 0..2 {
			self.fRec12[l19 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = F32::exp(-(2e+02 / self.fConst0));
		self.fConst2 = 1.0 - self.fConst1;
		self.fConst3 = F32::exp(-(33.333332 / self.fConst0));
		self.fConst4 = self.fConst0 * (1.0 - self.fConst3);
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		DelayDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("delay");
		ui_interface.add_horizontal_slider("a_time", ParamIndex(0), 0.333, 0.0, 1e+01, 0.0001);
		ui_interface.add_horizontal_slider("b_fb", ParamIndex(1), 0.6, 0.0, 1.0, 0.0001);
		ui_interface.add_horizontal_slider("c_type", ParamIndex(2), 0.0, 0.0, 3.0, 1.0);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			2 => Some(self.fHslider0),
			1 => Some(self.fHslider1),
			0 => Some(self.fHslider2),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			2 => { self.fHslider0 = value }
			1 => { self.fHslider1 = value }
			0 => { self.fHslider2 = value }
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
		let mut iSlow0: i32 = (self.fHslider0) as i32;
		let mut iSlow1: i32 = (iSlow0 >= 2) as i32;
		let mut iSlow2: i32 = (iSlow0 >= 1) as i32;
		let mut fSlow3: F32 = self.fConst2 * self.fHslider1;
		let mut fSlow4: F32 = self.fConst4 * self.fHslider2;
		let mut iSlow5: i32 = (iSlow0 >= 3) as i32;
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			self.fRec1[0] = fSlow3 + self.fConst1 * self.fRec1[1];
			let mut fTemp1: F32 = F32::max(0.0, F32::min(0.95, self.fRec1[0]));
			let mut fTemp2: F32 = fTemp0 + fTemp1 * self.fRec0[1];
			self.fVec0[(self.IOTA0 & 65535) as usize] = fTemp2;
			self.fRec2[0] = fSlow4 + self.fConst3 * self.fRec2[1];
			let mut fTemp3: F32 = F32::max(1.0, F32::min(65534.0, self.fRec2[0]));
			let mut iTemp4: i32 = (fTemp3) as i32;
			let mut iTemp5: i32 = std::cmp::min(65537, std::cmp::max(0, iTemp4));
			let mut fTemp6: F32 = F32::floor(fTemp3);
			let mut fTemp7: F32 = fTemp6 + (1.0 - fTemp3);
			let mut fTemp8: F32 = fTemp3 - fTemp6;
			let mut iTemp9: i32 = std::cmp::min(65537, std::cmp::max(0, i32::wrapping_add(iTemp4, 1)));
			self.fRec0[0] = self.fVec0[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] * fTemp7 + fTemp8 * self.fVec0[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			let mut fTemp10: F32 = *input1;
			let mut fTemp11: F32 = 0.5 * (fTemp0 + fTemp10) + fTemp1 * self.fRec4[1];
			self.fVec1[(self.IOTA0 & 65535) as usize] = fTemp11;
			self.fRec3[0] = fTemp7 * self.fVec1[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec1[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			let mut fTemp12: F32 = fTemp1 * self.fRec3[1];
			self.fVec2[(self.IOTA0 & 65535) as usize] = fTemp12;
			self.fRec4[0] = fTemp7 * self.fVec2[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec2[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			self.fRec6[0] = 0.65 * self.fRec6[1] + 0.35 * self.fRec5[1];
			let mut fTemp13: F32 = fTemp0 + fTemp1 * self.fRec6[0];
			self.fVec3[(self.IOTA0 & 65535) as usize] = fTemp13;
			self.fRec5[0] = fTemp7 * self.fVec3[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec3[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			let mut fTemp14: F32 = fTemp0 + 0.5 * self.fRec7[1];
			self.fVec4[(self.IOTA0 & 65535) as usize] = fTemp14;
			let mut fTemp15: F32 = fTemp7 * self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			self.fRec7[0] = fTemp15;
			let mut fTemp16: F32 = F32::max(1.0, fTemp3 * (0.167 * fTemp1 + 0.5));
			let mut iTemp17: i32 = (fTemp16) as i32;
			let mut iTemp18: i32 = std::cmp::min(65537, std::cmp::max(0, iTemp17));
			let mut fTemp19: F32 = F32::floor(fTemp16);
			let mut fTemp20: F32 = fTemp19 + (1.0 - fTemp16);
			let mut fTemp21: F32 = fTemp16 - fTemp19;
			let mut iTemp22: i32 = std::cmp::min(65537, std::cmp::max(0, i32::wrapping_add(iTemp17, 1)));
			let mut fTemp23: F32 = F32::max(1.0, fTemp3 * (0.083 * fTemp1 + 0.25));
			let mut iTemp24: i32 = (fTemp23) as i32;
			let mut iTemp25: i32 = std::cmp::min(65537, std::cmp::max(0, iTemp24));
			let mut fTemp26: F32 = F32::floor(fTemp23);
			let mut fTemp27: F32 = fTemp26 + (1.0 - fTemp23);
			let mut fTemp28: F32 = fTemp23 - fTemp26;
			let mut iTemp29: i32 = std::cmp::min(65537, std::cmp::max(0, i32::wrapping_add(iTemp24, 1)));
			let mut fTemp30: F32 = F32::max(1.0, fTemp3 * (0.042 * fTemp1 + 0.125));
			let mut iTemp31: i32 = (fTemp30) as i32;
			let mut iTemp32: i32 = std::cmp::min(65537, std::cmp::max(0, iTemp31));
			let mut fTemp33: F32 = F32::floor(fTemp30);
			let mut fTemp34: F32 = fTemp33 + (1.0 - fTemp30);
			let mut fTemp35: F32 = fTemp30 - fTemp33;
			let mut iTemp36: i32 = std::cmp::min(65537, std::cmp::max(0, i32::wrapping_add(iTemp31, 1)));
			let mut fRec8: F32 = fTemp15 + 0.7 * (self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp18)) & 65535) as usize] * fTemp20 + fTemp21 * self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp22)) & 65535) as usize]) + 0.5 * (self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp25)) & 65535) as usize] * fTemp27 + fTemp28 * self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp29)) & 65535) as usize]) + 0.35 * (self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp32)) & 65535) as usize] * fTemp34 + fTemp35 * self.fVec4[((i32::wrapping_sub(self.IOTA0, iTemp36)) & 65535) as usize]);
			*output0 = (if iSlow1 != 0 {(if iSlow5 != 0 {fRec8} else {self.fRec5[0]})} else {(if iSlow2 != 0 {self.fRec3[0]} else {self.fRec0[0]})});
			let mut fTemp37: F32 = fTemp10 + fTemp1 * self.fRec9[1];
			self.fVec5[(self.IOTA0 & 65535) as usize] = fTemp37;
			self.fRec9[0] = fTemp7 * self.fVec5[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec5[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			self.fRec11[0] = 0.65 * self.fRec11[1] + 0.35 * self.fRec10[1];
			let mut fTemp38: F32 = fTemp10 + fTemp1 * self.fRec11[0];
			self.fVec6[(self.IOTA0 & 65535) as usize] = fTemp38;
			self.fRec10[0] = fTemp7 * self.fVec6[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec6[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			let mut fTemp39: F32 = fTemp10 + 0.5 * self.fRec12[1];
			self.fVec7[(self.IOTA0 & 65535) as usize] = fTemp39;
			let mut fTemp40: F32 = fTemp7 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp5)) & 65535) as usize] + fTemp8 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp9)) & 65535) as usize];
			self.fRec12[0] = fTemp40;
			let mut fRec13: F32 = fTemp40 + 0.7 * (fTemp20 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp18)) & 65535) as usize] + fTemp21 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp22)) & 65535) as usize]) + 0.5 * (fTemp27 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp25)) & 65535) as usize] + fTemp28 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp29)) & 65535) as usize]) + 0.35 * (fTemp34 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp32)) & 65535) as usize] + fTemp35 * self.fVec7[((i32::wrapping_sub(self.IOTA0, iTemp36)) & 65535) as usize]);
			*output1 = (if iSlow1 != 0 {(if iSlow5 != 0 {fRec13} else {self.fRec10[0]})} else {(if iSlow2 != 0 {self.fRec4[0]} else {self.fRec9[0]})});
			self.fRec1[1] = self.fRec1[0];
			self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
			self.fRec2[1] = self.fRec2[0];
			self.fRec0[1] = self.fRec0[0];
			self.fRec3[1] = self.fRec3[0];
			self.fRec4[1] = self.fRec4[0];
			self.fRec6[1] = self.fRec6[0];
			self.fRec5[1] = self.fRec5[0];
			self.fRec7[1] = self.fRec7[0];
			self.fRec9[1] = self.fRec9[0];
			self.fRec11[1] = self.fRec11[0];
			self.fRec10[1] = self.fRec10[0];
			self.fRec12[1] = self.fRec12[0];
		}
		
	}

}

impl FaustDsp for DelayDsp {
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
