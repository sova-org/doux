/* ------------------------------------------------------------
name: "haas"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn HaasDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct HaasDsp {
	fSampleRate: i32,
	fConst0: F32,
	fHslider0: F32,
	IOTA0: i32,
	fVec0: [F32;8192],
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
pub const FAUST_ACTIVES: usize = 1;
pub const FAUST_PASSIVES: usize = 0;


impl HaasDsp {
		
	pub fn new() -> HaasDsp { 
		HaasDsp {
			fSampleRate: 0,
			fConst0: 0.0,
			fHslider0: 0.0,
			IOTA0: 0,
			fVec0: [0.0;8192],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("compile_options", r"-lang rust -ct 1 -cn HaasDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("delays.lib/fdelay4:author", r"Julius O. Smith III");
		m.declare("delays.lib/fdelayltv:author", r"Julius O. Smith III");
		m.declare("delays.lib/name", r"Faust Delay Library");
		m.declare("delays.lib/version", r"1.2.0");
		m.declare("filename", r"haas.dsp");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"haas");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.0;
	}
	pub fn instance_clear(&mut self) {
		self.IOTA0 = 0;
		for l0 in 0..8192 {
			self.fVec0[l0 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = 0.001 * F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		HaasDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("haas");
		ui_interface.add_horizontal_slider("a_ms", ParamIndex(0), 0.0, 0.0, 5e+01, 0.001);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			0 => Some(self.fHslider0),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			0 => { self.fHslider0 = value }
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
		let mut fSlow0: F32 = F32::max(2.0, F32::min(8189.0, self.fConst0 * self.fHslider0));
		let mut fSlow1: F32 = fSlow0 + -1.499995;
		let mut fSlow2: F32 = F32::floor(fSlow1);
		let mut fSlow3: F32 = fSlow0 + (-4.0 - fSlow2);
		let mut fSlow4: F32 = fSlow0 + (-3.0 - fSlow2);
		let mut fSlow5: F32 = fSlow0 + (-2.0 - fSlow2);
		let mut fSlow6: F32 = fSlow0 + (-1.0 - fSlow2);
		let mut fSlow7: F32 = 0.041666668 * fSlow6;
		let mut iSlow8: i32 = (fSlow1) as i32;
		let mut iSlow9: i32 = std::cmp::min(8192, std::cmp::max(0, iSlow8));
		let mut fSlow10: F32 = fSlow0 - fSlow2;
		let mut fSlow11: F32 = 0.16666667 * fSlow10;
		let mut iSlow12: i32 = std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iSlow8, 1)));
		let mut fSlow13: F32 = fSlow10 * fSlow6;
		let mut fSlow14: F32 = 0.25 * fSlow13;
		let mut iSlow15: i32 = std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iSlow8, 2)));
		let mut fSlow16: F32 = fSlow13 * fSlow5;
		let mut fSlow17: F32 = 0.16666667 * fSlow16;
		let mut iSlow18: i32 = std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iSlow8, 3)));
		let mut fSlow19: F32 = 0.041666668 * fSlow16 * fSlow4;
		let mut iSlow20: i32 = std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iSlow8, 4)));
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			self.fVec0[(self.IOTA0 & 8191) as usize] = fTemp0;
			*output0 = fSlow3 * (fSlow4 * (fSlow5 * (fSlow7 * self.fVec0[((i32::wrapping_sub(self.IOTA0, iSlow9)) & 8191) as usize] - fSlow11 * self.fVec0[((i32::wrapping_sub(self.IOTA0, iSlow12)) & 8191) as usize]) + fSlow14 * self.fVec0[((i32::wrapping_sub(self.IOTA0, iSlow15)) & 8191) as usize]) - fSlow17 * self.fVec0[((i32::wrapping_sub(self.IOTA0, iSlow18)) & 8191) as usize]) + fSlow19 * self.fVec0[((i32::wrapping_sub(self.IOTA0, iSlow20)) & 8191) as usize];
			self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
		}
		
	}

}

impl FaustDsp for HaasDsp {
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
