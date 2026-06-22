/* ------------------------------------------------------------
name: "flanger"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn FlangerDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct FlangerDsp {
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fHslider0: F32,
	fHslider1: F32,
	iVec1: [i32;2],
	fConst2: F32,
	fHslider2: F32,
	fRec2: [F32;2],
	fHslider3: F32,
	IOTA0: i32,
	fVec3: [F32;1024],
	fRec0: [F32;2],
}

pub type FaustFloat = F32;

pub struct FlangerDspSIG0 {
	iVec0: [i32;2],
	iRec1: [i32;2],
}

impl FlangerDspSIG0 {
	
	fn get_num_inputsFlangerDspSIG0(&self) -> i32 {
		return 0;
	}
	fn get_num_outputsFlangerDspSIG0(&self) -> i32 {
		return 1;
	}
	
	pub fn instance_initFlangerDspSIG0(&mut self, sample_rate: i32) {
		for l0 in 0..2 {
			self.iVec0[l0 as usize] = 0;
		}
		for l1 in 0..2 {
			self.iRec1[l1 as usize] = 0;
		}
	}
	
	pub fn fillFlangerDspSIG0(&mut self, count: i32, table: &mut[FaustFloat]) {
		for i1 in 0..count {
			self.iVec0[0] = 1;
			self.iRec1[0] = (i32::wrapping_add(self.iVec0[1], self.iRec1[1])) % 65536;
			table[i1 as usize] = F32::sin(9.58738e-05 * (self.iRec1[0]) as F32);
			self.iVec0[1] = self.iVec0[0];
			self.iRec1[1] = self.iRec1[0];
		}
	}

}


pub fn newFlangerDspSIG0() -> FlangerDspSIG0 { 
	FlangerDspSIG0 {
		iVec0: [0;2],
		iRec1: [0;2],
	}
}

pub struct FlangerDspSIG1 {
	iVec2: [i32;2],
	iRec3: [i32;2],
}

impl FlangerDspSIG1 {
	
	fn get_num_inputsFlangerDspSIG1(&self) -> i32 {
		return 0;
	}
	fn get_num_outputsFlangerDspSIG1(&self) -> i32 {
		return 1;
	}
	
	pub fn instance_initFlangerDspSIG1(&mut self, sample_rate: i32) {
		for l4 in 0..2 {
			self.iVec2[l4 as usize] = 0;
		}
		for l5 in 0..2 {
			self.iRec3[l5 as usize] = 0;
		}
	}
	
	pub fn fillFlangerDspSIG1(&mut self, count: i32, table: &mut[FaustFloat]) {
		for i2 in 0..count {
			self.iVec2[0] = 1;
			self.iRec3[0] = (i32::wrapping_add(self.iVec2[1], self.iRec3[1])) % 65536;
			table[i2 as usize] = F32::cos(9.58738e-05 * (self.iRec3[0]) as F32);
			self.iVec2[1] = self.iVec2[0];
			self.iRec3[1] = self.iRec3[0];
		}
	}

}


pub fn newFlangerDspSIG1() -> FlangerDspSIG1 { 
	FlangerDspSIG1 {
		iVec2: [0;2],
		iRec3: [0;2],
	}
}
fn FlangerDsp_faustpower2_f(value: F32) -> F32 {
	return value * value;
}
static ftbl0FlangerDspSIG0: std::sync::RwLock<[F32;65536]>  = std::sync::RwLock::new([0.0;65536]);
static ftbl1FlangerDspSIG1: std::sync::RwLock<[F32;65536]>  = std::sync::RwLock::new([0.0;65536]);
fn remainder_f32(from: f32, to: f32) -> f32 {
	from - to * (from / to).round_ties_even()
}
fn rint_f32(val: f32) -> f32 {
	val.round_ties_even()
}

pub const FAUST_INPUTS: usize = 1;
pub const FAUST_OUTPUTS: usize = 1;
pub const FAUST_ACTIVES: usize = 4;
pub const FAUST_PASSIVES: usize = 0;


impl FlangerDsp {
		
	pub fn new() -> FlangerDsp { 
		FlangerDsp {
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fHslider0: 0.0,
			fHslider1: 0.0,
			iVec1: [0;2],
			fConst2: 0.0,
			fHslider2: 0.0,
			fRec2: [0.0;2],
			fHslider3: 0.0,
			IOTA0: 0,
			fVec3: [0.0;1024],
			fRec0: [0.0;2],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/version", r"1.21.0");
		m.declare("compile_options", r"-lang rust -ct 1 -cn FlangerDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("delays.lib/fdelay4:author", r"Julius O. Smith III");
		m.declare("delays.lib/fdelayltv:author", r"Julius O. Smith III");
		m.declare("delays.lib/name", r"Faust Delay Library");
		m.declare("delays.lib/version", r"1.2.0");
		m.declare("filename", r"flanger.dsp");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"flanger");
		m.declare("oscillators.lib/name", r"Faust Oscillator Library");
		m.declare("oscillators.lib/version", r"1.6.0");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 2 static var(s)
		let mut ftbl0FlangerDspSIG0_guard = ftbl0FlangerDspSIG0.write().unwrap();
		let mut ftbl1FlangerDspSIG1_guard = ftbl1FlangerDspSIG1.write().unwrap();
		let mut sig0: FlangerDspSIG0 = newFlangerDspSIG0();
		sig0.instance_initFlangerDspSIG0(sample_rate);
		sig0.fillFlangerDspSIG0(65536, ftbl0FlangerDspSIG0_guard.as_mut());
		let mut sig1: FlangerDspSIG1 = newFlangerDspSIG1();
		sig1.instance_initFlangerDspSIG1(sample_rate);
		sig1.fillFlangerDspSIG1(65536, ftbl1FlangerDspSIG1_guard.as_mut());
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.7;
		self.fHslider1 = 0.0;
		self.fHslider2 = 0.0;
		self.fHslider3 = 0.35;
	}
	pub fn instance_clear(&mut self) {
		for l2 in 0..2 {
			self.iVec1[l2 as usize] = 0;
		}
		for l3 in 0..2 {
			self.fRec2[l3 as usize] = 0.0;
		}
		self.IOTA0 = 0;
		for l6 in 0..1024 {
			self.fVec3[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fRec0[l7 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 2 static var(s)
		let ftbl0FlangerDspSIG0_guard = ftbl0FlangerDspSIG0.read().unwrap();
		let ftbl1FlangerDspSIG1_guard = ftbl1FlangerDspSIG1.read().unwrap();
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = 0.001 * self.fConst0;
		self.fConst2 = 1.0 / self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		FlangerDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("flanger");
		ui_interface.add_horizontal_slider("a_rate", ParamIndex(0), 0.0, 0.0, 1e+02, 0.001);
		ui_interface.add_horizontal_slider("b_depth", ParamIndex(1), 0.7, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("c_fb", ParamIndex(2), 0.35, 0.0, 0.95, 0.001);
		ui_interface.add_horizontal_slider("d_phase", ParamIndex(3), 0.0, 0.0, 1.0, 0.001);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			1 => Some(self.fHslider0),
			3 => Some(self.fHslider1),
			0 => Some(self.fHslider2),
			2 => Some(self.fHslider3),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			1 => { self.fHslider0 = value }
			3 => { self.fHslider1 = value }
			0 => { self.fHslider2 = value }
			2 => { self.fHslider3 = value }
			_ => {}
		}
	}
	
	pub fn compute(
		&mut self,
		count: usize,
		inputs: &[impl AsRef<[FaustFloat]>],
		outputs: &mut[impl AsMut<[FaustFloat]>],
	) {
		
		// Obtaining locks on 2 static var(s)
		let ftbl0FlangerDspSIG0_guard = ftbl0FlangerDspSIG0.read().unwrap();
		let ftbl1FlangerDspSIG1_guard = ftbl1FlangerDspSIG1.read().unwrap();
		let [inputs0, .. ] = inputs.as_ref() else { panic!("wrong number of input buffers"); };
		let inputs0 = inputs0.as_ref()[..count].iter();
		let [outputs0, .. ] = outputs.as_mut() else { panic!("wrong number of output buffers"); };
		let outputs0 = outputs0.as_mut()[..count].iter_mut();
		let mut fSlow0: F32 = 4.75 * FlangerDsp_faustpower2_f(self.fHslider0);
		let mut fSlow1: F32 = 6.2831855 * self.fHslider1;
		let mut fSlow2: F32 = F32::cos(fSlow1);
		let mut fSlow3: F32 = self.fConst2 * self.fHslider2;
		let mut fSlow4: F32 = F32::sin(fSlow1);
		let mut fSlow5: F32 = F32::min(0.95, self.fHslider3);
		let zipped_iterators = inputs0.zip(outputs0);
		for (input0, output0) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			self.iVec1[0] = 1;
			let mut fTemp1: F32 = (if i32::wrapping_sub(1, self.iVec1[1]) != 0 {0.0} else {fSlow3 + self.fRec2[1]});
			self.fRec2[0] = fTemp1 - F32::floor(fTemp1);
			let mut iTemp2: i32 = std::cmp::max(0, std::cmp::min((65536.0 * self.fRec2[0]) as i32, 65535));
			let mut fTemp3: F32 = F32::max(2.0, F32::min(1021.0, self.fConst1 * (fSlow0 * (fSlow2 * ftbl0FlangerDspSIG0_guard[iTemp2 as usize] + fSlow4 * ftbl1FlangerDspSIG1_guard[iTemp2 as usize] + 1.0) + 0.5)));
			let mut fTemp4: F32 = fTemp3 + -1.499995;
			let mut fTemp5: F32 = F32::floor(fTemp4);
			let mut fTemp6: F32 = fTemp3 + (-3.0 - fTemp5);
			let mut fTemp7: F32 = fTemp3 + (-2.0 - fTemp5);
			let mut fTemp8: F32 = fTemp0 + fSlow5 * self.fRec0[1];
			self.fVec3[(self.IOTA0 & 1023) as usize] = fTemp8;
			let mut iTemp9: i32 = (fTemp4) as i32;
			let mut fTemp10: F32 = fTemp3 + (-1.0 - fTemp5);
			let mut fTemp11: F32 = fTemp3 - fTemp5;
			let mut fTemp12: F32 = fTemp11 * fTemp10;
			let mut fTemp13: F32 = fTemp12 * fTemp7;
			self.fRec0[0] = (fTemp3 + (-4.0 - fTemp5)) * (fTemp6 * (fTemp7 * (0.041666668 * self.fVec3[((i32::wrapping_sub(self.IOTA0, std::cmp::min(1024, std::cmp::max(0, iTemp9)))) & 1023) as usize] * fTemp10 - 0.16666667 * fTemp11 * self.fVec3[((i32::wrapping_sub(self.IOTA0, std::cmp::min(1024, std::cmp::max(0, i32::wrapping_add(iTemp9, 1))))) & 1023) as usize]) + 0.25 * fTemp12 * self.fVec3[((i32::wrapping_sub(self.IOTA0, std::cmp::min(1024, std::cmp::max(0, i32::wrapping_add(iTemp9, 2))))) & 1023) as usize]) - 0.16666667 * fTemp13 * self.fVec3[((i32::wrapping_sub(self.IOTA0, std::cmp::min(1024, std::cmp::max(0, i32::wrapping_add(iTemp9, 3))))) & 1023) as usize]) + 0.041666668 * fTemp13 * fTemp6 * self.fVec3[((i32::wrapping_sub(self.IOTA0, std::cmp::min(1024, std::cmp::max(0, i32::wrapping_add(iTemp9, 4))))) & 1023) as usize];
			*output0 = 0.5 * (fTemp0 + self.fRec0[0]);
			self.iVec1[1] = self.iVec1[0];
			self.fRec2[1] = self.fRec2[0];
			self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
			self.fRec0[1] = self.fRec0[0];
		}
		
	}

}

impl FaustDsp for FlangerDsp {
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
