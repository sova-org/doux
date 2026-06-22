/* ------------------------------------------------------------
name: "distort"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn DistortDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct DistortDsp {
	fHslider0: F32,
	fHslider1: F32,
	fHslider2: F32,
	fVec0: [F32;2],
	fSampleRate: i32,
}

pub type FaustFloat = F32;
fn DistortDsp_faustpower2_f(value: F32) -> F32 {
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
pub const FAUST_ACTIVES: usize = 3;
pub const FAUST_PASSIVES: usize = 0;


impl DistortDsp {
		
	pub fn new() -> DistortDsp { 
		DistortDsp {
			fHslider0: 0.0,
			fHslider1: 0.0,
			fHslider2: 0.0,
			fVec0: [0.0;2],
			fSampleRate: 0,
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("aanl.lib/ADAA1:author", r"Dario Sanfilippo");
		m.declare("aanl.lib/ADAA1:copyright", r"Copyright (C) 2021 Dario Sanfilippo     <sanfilippo.dario@gmail.com>");
		m.declare("aanl.lib/ADAA1:license", r"MIT License");
		m.declare("aanl.lib/arctan:author", r"Dario Sanfilippo");
		m.declare("aanl.lib/arctan:copyright", r"Copyright (C) 2021 Dario Sanfilippo     <sanfilippo.dario@gmail.com>");
		m.declare("aanl.lib/arctan:license", r"MIT License");
		m.declare("aanl.lib/hardclip:author", r"Dario Sanfilippo");
		m.declare("aanl.lib/hardclip:copyright", r"Copyright (C) 2021 Dario Sanfilippo     <sanfilippo.dario@gmail.com>");
		m.declare("aanl.lib/hardclip:license", r"MIT License");
		m.declare("aanl.lib/name", r"Faust Antialiased Nonlinearities");
		m.declare("aanl.lib/tanh1:author", r"Dario Sanfilippo");
		m.declare("aanl.lib/tanh1:copyright", r"Copyright (C) 2021 Dario Sanfilippo     <sanfilippo.dario@gmail.com>");
		m.declare("aanl.lib/tanh1:license", r"MIT License");
		m.declare("aanl.lib/version", r"1.4.1");
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/version", r"1.21.0");
		m.declare("compile_options", r"-lang rust -ct 1 -cn DistortDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("filename", r"distort.dsp");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"distort");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 1.0;
		self.fHslider1 = 0.0;
		self.fHslider2 = 0.0;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fVec0[l0 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		DistortDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("distort");
		ui_interface.add_horizontal_slider("a_distort", ParamIndex(0), 0.0, 0.0, 1e+02, 0.001);
		ui_interface.add_horizontal_slider("b_distortvol", ParamIndex(1), 1.0, 0.0, 2.0, 0.001);
		ui_interface.add_horizontal_slider("c_distortmode", ParamIndex(2), 0.0, 0.0, 3.0, 1.0);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			1 => Some(self.fHslider0),
			2 => Some(self.fHslider1),
			0 => Some(self.fHslider2),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			1 => { self.fHslider0 = value }
			2 => { self.fHslider1 = value }
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
		let [inputs0, .. ] = inputs.as_ref() else { panic!("wrong number of input buffers"); };
		let inputs0 = inputs0.as_ref()[..count].iter();
		let [outputs0, .. ] = outputs.as_mut() else { panic!("wrong number of output buffers"); };
		let outputs0 = outputs0.as_mut()[..count].iter_mut();
		let mut fSlow0: F32 = self.fHslider0;
		let mut iSlow1: i32 = (self.fHslider1) as i32;
		let mut iSlow2: i32 = (iSlow1 >= 2) as i32;
		let mut iSlow3: i32 = (iSlow1 >= 1) as i32;
		let mut fSlow4: F32 = F32::max(self.fHslider2, 0.0);
		let mut fSlow5: F32 = fSlow4 + 1.0;
		let mut fSlow6: F32 = F32::powf(fSlow5, -0.12);
		let mut fSlow7: F32 = fSlow5 * fSlow6;
		let mut fSlow8: F32 = 0.05 / (0.05 * fSlow4 + 1.0);
		let mut iSlow9: i32 = (iSlow1 >= 3) as i32;
		let mut fSlow10: F32 = 0.63661975 * fSlow6;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			let mut fTemp1: F32 = fTemp0 + 0.05;
			let mut fTemp2: F32 = fSlow5 * fTemp0;
			self.fVec0[0] = fTemp2;
			let mut fTemp3: F32 = fTemp2 - self.fVec0[1];
			let mut iTemp4: i32 = (F32::abs(fTemp3) <= 0.001) as i32;
			let mut fTemp5: F32 = 0.5 * (fTemp2 + self.fVec0[1]);
			let mut fTemp6: F32 = DistortDsp_faustpower2_f(fTemp2);
			let mut fTemp7: F32 = DistortDsp_faustpower2_f(self.fVec0[1]);
			*output0 = fSlow0 * (if iSlow2 != 0 {(if iSlow9 != 0 {fSlow6 * (if iTemp4 != 0 {F32::max(-1.0, F32::min(1.0, fTemp5))} else {((if ((fTemp2 <= 1.0) as i32) & ((fTemp2 >= -1.0) as i32) != 0 {0.5 * fTemp6} else {fSlow5 * fTemp0 * (((fTemp2 > 0.0) as i32) - ((fTemp2 < 0.0) as i32)) as F32 + -0.5}) - (if ((self.fVec0[1] <= 1.0) as i32) & ((self.fVec0[1] >= -1.0) as i32) != 0 {0.5 * fTemp7} else {self.fVec0[1] * (((self.fVec0[1] > 0.0) as i32) - ((self.fVec0[1] < 0.0) as i32)) as F32 + -0.5})) / fTemp3})} else {fSlow10 * (if iTemp4 != 0 {F32::atan(fTemp5)} else {(fSlow5 * fTemp0 * F32::atan(fTemp2) - (self.fVec0[1] * F32::atan(self.fVec0[1]) + 0.5 * (F32::log(fTemp6 + 1.0, std::f32::consts::E) - F32::log(fTemp7 + 1.0, std::f32::consts::E)))) / fTemp3})})} else {(if iSlow3 != 0 {fSlow6 * (if iTemp4 != 0 {F32::tanh(fTemp5)} else {(F32::log(F32::min(3.4028235e+38, F32::cosh(fTemp2)), std::f32::consts::E) - F32::log(F32::min(3.4028235e+38, F32::cosh(self.fVec0[1])), std::f32::consts::E)) / fTemp3})} else {fSlow7 * (fTemp1 / (fSlow4 * F32::abs(fTemp1) + 1.0) - fSlow8)})});
			self.fVec0[1] = self.fVec0[0];
		}
		
	}

}

impl FaustDsp for DistortDsp {
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
