/* ------------------------------------------------------------
name: "phaser"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn PhaserDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct PhaserDsp {
	fHslider0: F32,
	iVec0: [i32;2],
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fConst2: F32,
	fConst3: F32,
	fHslider1: F32,
	fConst4: F32,
	fHslider2: F32,
	fHslider3: F32,
	fConst5: F32,
	fHslider4: F32,
	fRec5: [F32;2],
	fRec6: [F32;2],
	fConst6: F32,
	fRec4: [F32;3],
	fConst7: F32,
	fRec3: [F32;3],
	fConst8: F32,
	fRec2: [F32;3],
	fConst9: F32,
	fRec1: [F32;3],
	fRec0: [F32;2],
}

pub type FaustFloat = F32;
fn PhaserDsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
fn remainder_f32(from: f32, to: f32) -> f32 {
	from - to * (from / to).round_ties_even()
}
fn rint_f32(val: f32) -> f32 {
	val.round_ties_even()
}

pub const FAUST_INPUTS: usize = 1;
pub const FAUST_OUTPUTS: usize = 1;
pub const FAUST_ACTIVES: usize = 5;
pub const FAUST_PASSIVES: usize = 0;


impl PhaserDsp {
		
	pub fn new() -> PhaserDsp { 
		PhaserDsp {
			fHslider0: 0.0,
			iVec0: [0;2],
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fConst2: 0.0,
			fConst3: 0.0,
			fHslider1: 0.0,
			fConst4: 0.0,
			fHslider2: 0.0,
			fHslider3: 0.0,
			fConst5: 0.0,
			fHslider4: 0.0,
			fRec5: [0.0;2],
			fRec6: [0.0;2],
			fConst6: 0.0,
			fRec4: [0.0;3],
			fConst7: 0.0,
			fRec3: [0.0;3],
			fConst8: 0.0,
			fRec2: [0.0;3],
			fConst9: 0.0,
			fRec1: [0.0;3],
			fRec0: [0.0;2],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("compile_options", r"-lang rust -ct 1 -cn PhaserDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("filename", r"phaser.dsp");
		m.declare("filters.lib/fir:author", r"Julius O. Smith III");
		m.declare("filters.lib/fir:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/fir:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/iir:author", r"Julius O. Smith III");
		m.declare("filters.lib/iir:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/iir:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/lowpass0_highpass1", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/name", r"Faust Filters Library");
		m.declare("filters.lib/nlf2:author", r"Julius O. Smith III");
		m.declare("filters.lib/nlf2:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/nlf2:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/tf2:author", r"Julius O. Smith III");
		m.declare("filters.lib/tf2:copyright", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/tf2:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/version", r"1.7.1");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"phaser");
		m.declare("oscillators.lib/name", r"Faust Oscillator Library");
		m.declare("oscillators.lib/version", r"1.6.0");
		m.declare("phaflangers.lib/name", r"Faust Phaser and Flanger Library");
		m.declare("phaflangers.lib/version", r"1.1.0");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.0;
		self.fHslider1 = 8e+02;
		self.fHslider2 = 1.2e+03;
		self.fHslider3 = 0.0;
		self.fHslider4 = 0.0;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.iVec0[l0 as usize] = 0;
		}
		for l1 in 0..2 {
			self.fRec5[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec6[l2 as usize] = 0.0;
		}
		for l3 in 0..3 {
			self.fRec4[l3 as usize] = 0.0;
		}
		for l4 in 0..3 {
			self.fRec3[l4 as usize] = 0.0;
		}
		for l5 in 0..3 {
			self.fRec2[l5 as usize] = 0.0;
		}
		for l6 in 0..3 {
			self.fRec1[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fRec0[l7 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = F32::exp(-(3141.5928 / self.fConst0));
		self.fConst2 = 2.0 * self.fConst1;
		self.fConst3 = 1.5 / self.fConst0;
		self.fConst4 = 0.45 * self.fConst0;
		self.fConst5 = 6.2831855 / self.fConst0;
		self.fConst6 = PhaserDsp_faustpower2_f(self.fConst1);
		self.fConst7 = 2.25 / self.fConst0;
		self.fConst8 = 3.375 / self.fConst0;
		self.fConst9 = 5.0625 / self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		PhaserDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("phaser");
		ui_interface.add_horizontal_slider("a_speed", ParamIndex(0), 0.0, 0.0, 1e+02, 0.001);
		ui_interface.add_horizontal_slider("b_fb", ParamIndex(1), 0.0, 0.0, 0.95, 0.001);
		ui_interface.add_horizontal_slider("c_sweep", ParamIndex(2), 1.2e+03, 0.0, 2e+04, 0.001);
		ui_interface.add_horizontal_slider("d_center", ParamIndex(3), 8e+02, 0.0, 2e+04, 0.001);
		ui_interface.add_horizontal_slider("e_phase", ParamIndex(4), 0.0, 0.0, 1.0, 0.001);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			1 => Some(self.fHslider0),
			3 => Some(self.fHslider1),
			2 => Some(self.fHslider2),
			4 => Some(self.fHslider3),
			0 => Some(self.fHslider4),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			1 => { self.fHslider0 = value }
			3 => { self.fHslider1 = value }
			2 => { self.fHslider2 = value }
			4 => { self.fHslider3 = value }
			0 => { self.fHslider4 = value }
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
		let [inputs0, .. ] = inputs.as_ref() else { panic!("wrong number of input buffers"); };
		let inputs0 = inputs0.as_ref()[..count].iter();
		let [outputs0, .. ] = outputs.as_mut() else { panic!("wrong number of output buffers"); };
		let outputs0 = outputs0.as_mut()[..count].iter_mut();
		let mut fSlow0: F32 = self.fHslider0;
		let mut fSlow1: F32 = F32::max(2e+01, self.fHslider1);
		let mut fSlow2: F32 = 6.2831855 * fSlow1;
		let mut fSlow3: F32 = 3.1415927 * (fSlow1 - F32::min(self.fConst4, fSlow1 * F32::powf(2.0, 0.00083333335 * self.fHslider2)));
		let mut fSlow4: F32 = self.fHslider3;
		let mut fSlow5: F32 = self.fConst5 * self.fHslider4;
		let mut fSlow6: F32 = F32::sin(fSlow5);
		let mut fSlow7: F32 = F32::cos(fSlow5);
		let mut fSlow8: F32 = 1.0 - fSlow4;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			self.iVec0[0] = 1;
			self.fRec5[0] = fSlow6 * self.fRec6[1] + fSlow7 * self.fRec5[1];
			self.fRec6[0] = (i32::wrapping_sub(1, self.iVec0[1])) as F32 + fSlow7 * self.fRec6[1] - fSlow6 * self.fRec5[1];
			let mut fTemp1: F32 = fSlow2 - fSlow3 * (1.0 - (fSlow4 * self.fRec6[0] + fSlow8 * self.fRec5[0]));
			let mut fTemp2: F32 = self.fRec4[1] * F32::cos(self.fConst3 * fTemp1);
			self.fRec4[0] = fTemp0 + fSlow0 * self.fRec0[1] + self.fConst2 * fTemp2 - self.fConst6 * self.fRec4[2];
			let mut fTemp3: F32 = self.fRec3[1] * F32::cos(self.fConst7 * fTemp1);
			self.fRec3[0] = self.fRec4[2] + self.fConst6 * (self.fRec4[0] - self.fRec3[2]) - self.fConst2 * (fTemp2 - fTemp3);
			let mut fTemp4: F32 = self.fRec2[1] * F32::cos(self.fConst8 * fTemp1);
			self.fRec2[0] = self.fRec3[2] + self.fConst6 * (self.fRec3[0] - self.fRec2[2]) - self.fConst2 * (fTemp3 - fTemp4);
			let mut fTemp5: F32 = self.fRec1[1] * F32::cos(self.fConst9 * fTemp1);
			self.fRec1[0] = self.fRec2[2] + self.fConst6 * (self.fRec2[0] - self.fRec1[2]) - self.fConst2 * (fTemp4 - fTemp5);
			self.fRec0[0] = self.fRec1[2] + self.fConst6 * self.fRec1[0] - self.fConst2 * fTemp5;
			*output0 = 0.5 * (fTemp0 + self.fRec0[0]);
			self.iVec0[1] = self.iVec0[0];
			self.fRec5[1] = self.fRec5[0];
			self.fRec6[1] = self.fRec6[0];
			self.fRec4[2] = self.fRec4[1];
			self.fRec4[1] = self.fRec4[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fRec2[2] = self.fRec2[1];
			self.fRec2[1] = self.fRec2[0];
			self.fRec1[2] = self.fRec1[1];
			self.fRec1[1] = self.fRec1[0];
			self.fRec0[1] = self.fRec0[0];
		}
		
	}

}

impl FaustDsp for PhaserDsp {
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
