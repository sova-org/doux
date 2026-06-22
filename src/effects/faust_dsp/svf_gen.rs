/* ------------------------------------------------------------
name: "svf"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn SvfDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct SvfDsp {
	fHslider0: F32,
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fHslider1: F32,
	fConst2: F32,
	fHslider2: F32,
	fRec0: [F32;2],
	fRec1: [F32;2],
}

pub type FaustFloat = F32;
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


impl SvfDsp {
		
	pub fn new() -> SvfDsp { 
		SvfDsp {
			fHslider0: 0.0,
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fHslider1: 0.0,
			fConst2: 0.0,
			fHslider2: 0.0,
			fRec0: [0.0;2],
			fRec1: [0.0;2],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("compile_options", r"-lang rust -ct 1 -cn SvfDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("filename", r"svf.dsp");
		m.declare("filters.lib/lowpass0_highpass1", r"Copyright (C) 2003-2019 by Julius O. Smith III <jos@ccrma.stanford.edu>");
		m.declare("filters.lib/name", r"Faust Filters Library");
		m.declare("filters.lib/svf:author", r"Oleg Nesterov");
		m.declare("filters.lib/svf:copyright", r"Copyright (C) 2020 Oleg Nesterov <oleg@redhat.com>");
		m.declare("filters.lib/svf:license", r"MIT-style STK-4.3 license");
		m.declare("filters.lib/version", r"1.7.1");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"svf");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
		m.declare("routes.lib/name", r"Faust Signal Routing Library");
		m.declare("routes.lib/version", r"1.2.0");
		m.declare("signals.lib/name", r"Faust Signal Routing Library");
		m.declare("signals.lib/version", r"1.6.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.0;
		self.fHslider1 = 1e+03;
		self.fHslider2 = 0.0;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fRec0[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec1[l1 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = 3.1415927 / self.fConst0;
		self.fConst2 = 0.45 * self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		SvfDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("svf");
		ui_interface.add_horizontal_slider("a_cutoff", ParamIndex(0), 1e+03, 1.0, 2e+04, 0.001);
		ui_interface.add_horizontal_slider("b_q", ParamIndex(1), 0.0, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("c_mode", ParamIndex(2), 0.0, 0.0, 2.0, 1.0);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			2 => Some(self.fHslider0),
			0 => Some(self.fHslider1),
			1 => Some(self.fHslider2),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			2 => { self.fHslider0 = value }
			0 => { self.fHslider1 = value }
			1 => { self.fHslider2 = value }
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
		let mut iSlow0: i32 = (self.fHslider0) as i32;
		let mut iSlow1: i32 = (iSlow0 == 0) as i32;
		let mut iSlow2: i32 = (iSlow0 == 1) as i32;
		let mut fSlow3: F32 = F32::tan(self.fConst1 * F32::max(1.0, F32::min(self.fHslider1, self.fConst2)));
		let mut fSlow4: F32 = 1.0 / (3e+01 * self.fHslider2 + 0.5);
		let mut fSlow5: F32 = fSlow3 * (fSlow3 + fSlow4) + 1.0;
		let mut fSlow6: F32 = 2.0 / fSlow5;
		let mut fSlow7: F32 = fSlow3 / fSlow5;
		let mut fSlow8: F32 = 1.0 / fSlow5;
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			let mut fTemp1: F32 = self.fRec0[1] + fSlow3 * (fTemp0 - self.fRec1[1]);
			self.fRec0[0] = fSlow6 * fTemp1 - self.fRec0[1];
			let mut fTemp2: F32 = self.fRec1[1] + fSlow7 * fTemp1;
			self.fRec1[0] = 2.0 * fTemp2 - self.fRec1[1];
			let mut fRec2: F32 = fSlow8 * fTemp1;
			let mut fRec3: F32 = fTemp2;
			*output0 = (if iSlow1 != 0 {fRec3} else {(if iSlow2 != 0 {fTemp0 - (fRec3 + fSlow4 * fRec2)} else {fRec2})});
			self.fRec0[1] = self.fRec0[0];
			self.fRec1[1] = self.fRec1[0];
		}
		
	}

}

impl FaustDsp for SvfDsp {
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
