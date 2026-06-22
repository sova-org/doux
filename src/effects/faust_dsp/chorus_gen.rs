/* ------------------------------------------------------------
name: "chorus"
Code generated with Faust 2.81.2 (https://faust.grame.fr)
Compilation options: -lang rust -ct 1 -cn ChorusDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0
------------------------------------------------------------ */
#[repr(C)]
pub struct ChorusDsp {
	fHslider0: F32,
	iVec0: [i32;2],
	fSampleRate: i32,
	fConst0: F32,
	fConst1: F32,
	fConst2: F32,
	fHslider1: F32,
	fRec0: [F32;2],
	fHslider2: F32,
	fConst3: F32,
	fHslider3: F32,
	fRec2: [F32;2],
	IOTA0: i32,
	fVec2: [F32;8192],
}

pub type FaustFloat = F32;

pub struct ChorusDspSIG0 {
	iVec1: [i32;2],
	iRec1: [i32;2],
}

impl ChorusDspSIG0 {
	
	fn get_num_inputsChorusDspSIG0(&self) -> i32 {
		return 0;
	}
	fn get_num_outputsChorusDspSIG0(&self) -> i32 {
		return 1;
	}
	
	pub fn instance_initChorusDspSIG0(&mut self, sample_rate: i32) {
		for l2 in 0..2 {
			self.iVec1[l2 as usize] = 0;
		}
		for l3 in 0..2 {
			self.iRec1[l3 as usize] = 0;
		}
	}
	
	pub fn fillChorusDspSIG0(&mut self, count: i32, table: &mut[FaustFloat]) {
		for i1 in 0..count {
			self.iVec1[0] = 1;
			self.iRec1[0] = (i32::wrapping_add(self.iVec1[1], self.iRec1[1])) % 65536;
			table[i1 as usize] = F32::sin(9.58738e-05 * (self.iRec1[0]) as F32);
			self.iVec1[1] = self.iVec1[0];
			self.iRec1[1] = self.iRec1[0];
		}
	}

}


pub fn newChorusDspSIG0() -> ChorusDspSIG0 { 
	ChorusDspSIG0 {
		iVec1: [0;2],
		iRec1: [0;2],
	}
}

pub struct ChorusDspSIG1 {
	iVec3: [i32;2],
	iRec3: [i32;2],
}

impl ChorusDspSIG1 {
	
	fn get_num_inputsChorusDspSIG1(&self) -> i32 {
		return 0;
	}
	fn get_num_outputsChorusDspSIG1(&self) -> i32 {
		return 1;
	}
	
	pub fn instance_initChorusDspSIG1(&mut self, sample_rate: i32) {
		for l6 in 0..2 {
			self.iVec3[l6 as usize] = 0;
		}
		for l7 in 0..2 {
			self.iRec3[l7 as usize] = 0;
		}
	}
	
	pub fn fillChorusDspSIG1(&mut self, count: i32, table: &mut[FaustFloat]) {
		for i2 in 0..count {
			self.iVec3[0] = 1;
			self.iRec3[0] = (i32::wrapping_add(self.iVec3[1], self.iRec3[1])) % 65536;
			table[i2 as usize] = F32::cos(9.58738e-05 * (self.iRec3[0]) as F32);
			self.iVec3[1] = self.iVec3[0];
			self.iRec3[1] = self.iRec3[0];
		}
	}

}


pub fn newChorusDspSIG1() -> ChorusDspSIG1 { 
	ChorusDspSIG1 {
		iVec3: [0;2],
		iRec3: [0;2],
	}
}
static ftbl0ChorusDspSIG0: std::sync::RwLock<[F32;65536]>  = std::sync::RwLock::new([0.0;65536]);
static ftbl1ChorusDspSIG1: std::sync::RwLock<[F32;65536]>  = std::sync::RwLock::new([0.0;65536]);
fn remainder_f32(from: f32, to: f32) -> f32 {
	from - to * (from / to).round_ties_even()
}
fn rint_f32(val: f32) -> f32 {
	val.round_ties_even()
}

pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 4;
pub const FAUST_PASSIVES: usize = 0;


impl ChorusDsp {
		
	pub fn new() -> ChorusDsp { 
		ChorusDsp {
			fHslider0: 0.0,
			iVec0: [0;2],
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fConst2: 0.0,
			fHslider1: 0.0,
			fRec0: [0.0;2],
			fHslider2: 0.0,
			fConst3: 0.0,
			fHslider3: 0.0,
			fRec2: [0.0;2],
			IOTA0: 0,
			fVec2: [0.0;8192],
		}
	}
	pub fn metadata(&self, m: &mut dyn Meta) { 
		m.declare("basics.lib/name", r"Faust Basic Element Library");
		m.declare("basics.lib/version", r"1.21.0");
		m.declare("compile_options", r"-lang rust -ct 1 -cn ChorusDsp -es 1 -mcd 16 -mdd 1024 -mdy 33 -single -ftz 0");
		m.declare("delays.lib/fdelay4:author", r"Julius O. Smith III");
		m.declare("delays.lib/fdelayltv:author", r"Julius O. Smith III");
		m.declare("delays.lib/name", r"Faust Delay Library");
		m.declare("delays.lib/version", r"1.2.0");
		m.declare("filename", r"chorus.dsp");
		m.declare("maths.lib/author", r"GRAME");
		m.declare("maths.lib/copyright", r"GRAME");
		m.declare("maths.lib/license", r"LGPL with exception");
		m.declare("maths.lib/name", r"Faust Math Library");
		m.declare("maths.lib/version", r"2.8.1");
		m.declare("name", r"chorus");
		m.declare("oscillators.lib/name", r"Faust Oscillator Library");
		m.declare("oscillators.lib/version", r"1.6.0");
		m.declare("platform.lib/name", r"Generic Platform Library");
		m.declare("platform.lib/version", r"1.3.0");
		m.declare("signals.lib/name", r"Faust Signal Routing Library");
		m.declare("signals.lib/version", r"1.6.0");
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 2 static var(s)
		let mut ftbl0ChorusDspSIG0_guard = ftbl0ChorusDspSIG0.write().unwrap();
		let mut ftbl1ChorusDspSIG1_guard = ftbl1ChorusDspSIG1.write().unwrap();
		let mut sig0: ChorusDspSIG0 = newChorusDspSIG0();
		sig0.instance_initChorusDspSIG0(sample_rate);
		sig0.fillChorusDspSIG0(65536, ftbl0ChorusDspSIG0_guard.as_mut());
		let mut sig1: ChorusDspSIG1 = newChorusDspSIG1();
		sig1.instance_initChorusDspSIG1(sample_rate);
		sig1.fillChorusDspSIG1(65536, ftbl1ChorusDspSIG1_guard.as_mut());
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = 0.0;
		self.fHslider1 = 25.0;
		self.fHslider2 = 0.35;
		self.fHslider3 = 0.0;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.iVec0[l0 as usize] = 0;
		}
		for l1 in 0..2 {
			self.fRec0[l1 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fRec2[l4 as usize] = 0.0;
		}
		self.IOTA0 = 0;
		for l5 in 0..8192 {
			self.fVec2[l5 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 2 static var(s)
		let ftbl0ChorusDspSIG0_guard = ftbl0ChorusDspSIG0.read().unwrap();
		let ftbl1ChorusDspSIG1_guard = ftbl1ChorusDspSIG1.read().unwrap();
		self.fSampleRate = sample_rate;
		self.fConst0 = F32::min(1.92e+05, F32::max(1.0, (self.fSampleRate) as F32));
		self.fConst1 = F32::exp(-(5e+01 / self.fConst0));
		self.fConst2 = 0.001 * self.fConst0 * (1.0 - self.fConst1);
		self.fConst3 = 1.0 / self.fConst0;
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		ChorusDsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	pub fn build_user_interface(&self, ui_interface: &mut dyn UI<FaustFloat>) {
		Self::build_user_interface_static(ui_interface);
	}
	
	pub fn build_user_interface_static(ui_interface: &mut dyn UI<FaustFloat>) {
		ui_interface.open_vertical_box("chorus");
		ui_interface.add_horizontal_slider("a_rate", ParamIndex(0), 0.0, 0.0, 1e+02, 0.001);
		ui_interface.add_horizontal_slider("b_depth", ParamIndex(1), 0.35, 0.0, 1.0, 0.001);
		ui_interface.add_horizontal_slider("c_delay", ParamIndex(2), 25.0, 0.0, 1e+02, 0.001);
		ui_interface.add_horizontal_slider("d_type", ParamIndex(3), 0.0, 0.0, 2.0, 1.0);
		ui_interface.close_box();
	}
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			3 => Some(self.fHslider0),
			2 => Some(self.fHslider1),
			1 => Some(self.fHslider2),
			0 => Some(self.fHslider3),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			3 => { self.fHslider0 = value }
			2 => { self.fHslider1 = value }
			1 => { self.fHslider2 = value }
			0 => { self.fHslider3 = value }
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
		let ftbl0ChorusDspSIG0_guard = ftbl0ChorusDspSIG0.read().unwrap();
		let ftbl1ChorusDspSIG1_guard = ftbl1ChorusDspSIG1.read().unwrap();
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
		let mut fSlow4: F32 = self.fHslider2;
		let mut fSlow5: F32 = 0.8 * fSlow4;
		let mut fSlow6: F32 = self.fConst3 * self.fHslider3;
		let mut fSlow7: F32 = 1.2 * fSlow4;
		let mut fSlow8: F32 = 1.6 * fSlow4;
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			let mut fTemp0: F32 = *input0;
			self.iVec0[0] = 1;
			self.fRec0[0] = fSlow3 + self.fConst1 * self.fRec0[1];
			let mut fTemp1: F32 = (if i32::wrapping_sub(1, self.iVec0[1]) != 0 {0.0} else {fSlow6 + self.fRec2[1]});
			self.fRec2[0] = fTemp1 - F32::floor(fTemp1);
			let mut iTemp2: i32 = std::cmp::max(0, std::cmp::min((65536.0 * self.fRec2[0]) as i32, 65535));
			let mut fTemp3: F32 = ftbl0ChorusDspSIG0_guard[iTemp2 as usize];
			let mut fTemp4: F32 = fSlow5 * fTemp3;
			let mut fTemp5: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (fTemp4 + 1.0)));
			let mut fTemp6: F32 = fTemp5 + -1.499995;
			let mut fTemp7: F32 = F32::floor(fTemp6);
			let mut fTemp8: F32 = fTemp5 - fTemp7;
			let mut fTemp9: F32 = fTemp5 + (-1.0 - fTemp7);
			let mut fTemp10: F32 = fTemp8 * fTemp9;
			let mut fTemp11: F32 = fTemp5 + (-2.0 - fTemp7);
			let mut fTemp12: F32 = fTemp10 * fTemp11;
			let mut fTemp13: F32 = fTemp5 + (-3.0 - fTemp7);
			let mut fTemp14: F32 = *input1;
			let mut fTemp15: F32 = fTemp0 + fTemp14;
			self.fVec2[(self.IOTA0 & 8191) as usize] = fTemp15;
			let mut iTemp16: i32 = (fTemp6) as i32;
			let mut fTemp17: F32 = ftbl1ChorusDspSIG1_guard[iTemp2 as usize];
			let mut fTemp18: F32 = fSlow5 * (0.8660254 * fTemp17 - 0.5 * fTemp3);
			let mut fTemp19: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (fTemp18 + 1.0)));
			let mut fTemp20: F32 = fTemp19 + -1.499995;
			let mut fTemp21: F32 = F32::floor(fTemp20);
			let mut fTemp22: F32 = fTemp19 - fTemp21;
			let mut fTemp23: F32 = fTemp19 + (-1.0 - fTemp21);
			let mut fTemp24: F32 = fTemp22 * fTemp23;
			let mut fTemp25: F32 = fTemp19 + (-2.0 - fTemp21);
			let mut fTemp26: F32 = fTemp24 * fTemp25;
			let mut fTemp27: F32 = fTemp19 + (-3.0 - fTemp21);
			let mut iTemp28: i32 = (fTemp20) as i32;
			let mut fTemp29: F32 = fSlow5 * (0.5 * fTemp3 + 0.8660254 * fTemp17);
			let mut fTemp30: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (1.0 - fTemp29)));
			let mut fTemp31: F32 = fTemp30 + -1.499995;
			let mut fTemp32: F32 = F32::floor(fTemp31);
			let mut fTemp33: F32 = fTemp30 - fTemp32;
			let mut fTemp34: F32 = fTemp30 + (-1.0 - fTemp32);
			let mut fTemp35: F32 = fTemp33 * fTemp34;
			let mut fTemp36: F32 = fTemp30 + (-2.0 - fTemp32);
			let mut fTemp37: F32 = fTemp35 * fTemp36;
			let mut fTemp38: F32 = fTemp30 + (-3.0 - fTemp32);
			let mut iTemp39: i32 = (fTemp31) as i32;
			let mut fTemp40: F32 = fSlow7 * fTemp3;
			let mut fTemp41: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (fTemp40 + 1.0)));
			let mut fTemp42: F32 = fTemp41 + -1.499995;
			let mut fTemp43: F32 = F32::floor(fTemp42);
			let mut fTemp44: F32 = fTemp41 - fTemp43;
			let mut fTemp45: F32 = fTemp41 + (-1.0 - fTemp43);
			let mut fTemp46: F32 = fTemp44 * fTemp45;
			let mut fTemp47: F32 = fTemp41 + (-2.0 - fTemp43);
			let mut fTemp48: F32 = fTemp46 * fTemp47;
			let mut fTemp49: F32 = fTemp41 + (-3.0 - fTemp43);
			let mut iTemp50: i32 = (fTemp42) as i32;
			let mut fTemp51: F32 = fTemp48 * fTemp49 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp50, 4))))) & 8191) as usize];
			let mut fTemp52: F32 = fSlow7 * fTemp17;
			let mut fTemp53: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (fTemp52 + 1.0)));
			let mut fTemp54: F32 = fTemp53 + -1.499995;
			let mut fTemp55: F32 = F32::floor(fTemp54);
			let mut fTemp56: F32 = fTemp53 - fTemp55;
			let mut fTemp57: F32 = fTemp53 + (-1.0 - fTemp55);
			let mut fTemp58: F32 = fTemp56 * fTemp57;
			let mut fTemp59: F32 = fTemp53 + (-2.0 - fTemp55);
			let mut fTemp60: F32 = fTemp58 * fTemp59;
			let mut fTemp61: F32 = fTemp53 + (-3.0 - fTemp55);
			let mut iTemp62: i32 = (fTemp54) as i32;
			let mut fTemp63: F32 = fTemp60 * fTemp61 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp62, 4))))) & 8191) as usize];
			let mut fTemp64: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (1.0 - fTemp40)));
			let mut fTemp65: F32 = fTemp64 + -1.499995;
			let mut fTemp66: F32 = F32::floor(fTemp65);
			let mut fTemp67: F32 = fTemp64 - fTemp66;
			let mut fTemp68: F32 = fTemp64 + (-1.0 - fTemp66);
			let mut fTemp69: F32 = fTemp67 * fTemp68;
			let mut fTemp70: F32 = fTemp64 + (-2.0 - fTemp66);
			let mut fTemp71: F32 = fTemp69 * fTemp70;
			let mut fTemp72: F32 = fTemp64 + (-3.0 - fTemp66);
			let mut iTemp73: i32 = (fTemp65) as i32;
			let mut fTemp74: F32 = fTemp71 * fTemp72 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp73, 4))))) & 8191) as usize];
			let mut fTemp75: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (1.0 - fTemp52)));
			let mut fTemp76: F32 = fTemp75 + -1.499995;
			let mut fTemp77: F32 = F32::floor(fTemp76);
			let mut fTemp78: F32 = fTemp75 - fTemp77;
			let mut fTemp79: F32 = fTemp75 + (-1.0 - fTemp77);
			let mut fTemp80: F32 = fTemp78 * fTemp79;
			let mut fTemp81: F32 = fTemp75 + (-2.0 - fTemp77);
			let mut fTemp82: F32 = fTemp80 * fTemp81;
			let mut fTemp83: F32 = fTemp75 + (-3.0 - fTemp77);
			let mut iTemp84: i32 = (fTemp76) as i32;
			let mut fTemp85: F32 = fTemp82 * fTemp83 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp84, 4))))) & 8191) as usize];
			let mut fTemp86: F32 = (fTemp41 + (-4.0 - fTemp43)) * (fTemp49 * (fTemp47 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp50)))) & 8191) as usize] * fTemp45 - 0.083333336 * fTemp44 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp50, 1))))) & 8191) as usize]) + 0.125 * fTemp46 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp50, 2))))) & 8191) as usize]) - 0.083333336 * fTemp48 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp50, 3))))) & 8191) as usize]) + (fTemp53 + (-4.0 - fTemp55)) * (fTemp61 * (fTemp59 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp62)))) & 8191) as usize] * fTemp57 - 0.083333336 * fTemp56 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp62, 1))))) & 8191) as usize]) + 0.125 * fTemp58 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp62, 2))))) & 8191) as usize]) - 0.083333336 * fTemp60 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp62, 3))))) & 8191) as usize]) + (fTemp64 + (-4.0 - fTemp66)) * (fTemp72 * (fTemp70 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp73)))) & 8191) as usize] * fTemp68 - 0.083333336 * fTemp67 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp73, 1))))) & 8191) as usize]) + 0.125 * fTemp69 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp73, 2))))) & 8191) as usize]) - 0.083333336 * fTemp71 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp73, 3))))) & 8191) as usize]) + (fTemp75 + (-4.0 - fTemp77)) * (fTemp83 * (fTemp81 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp84)))) & 8191) as usize] * fTemp79 - 0.083333336 * fTemp78 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp84, 1))))) & 8191) as usize]) + 0.125 * fTemp80 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp84, 2))))) & 8191) as usize]) - 0.083333336 * fTemp82 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp84, 3))))) & 8191) as usize]);
			let mut fTemp87: F32 = fSlow8 * fTemp17;
			let mut fTemp88: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (fTemp87 + 1.0)));
			let mut fTemp89: F32 = fTemp88 + -1.499995;
			let mut fTemp90: F32 = F32::floor(fTemp89);
			let mut fTemp91: F32 = fTemp88 - fTemp90;
			let mut fTemp92: F32 = fTemp88 + (-1.0 - fTemp90);
			let mut fTemp93: F32 = fTemp91 * fTemp92;
			let mut fTemp94: F32 = fTemp88 + (-2.0 - fTemp90);
			let mut fTemp95: F32 = fTemp93 * fTemp94;
			let mut fTemp96: F32 = fTemp88 + (-3.0 - fTemp90);
			let mut iTemp97: i32 = (fTemp89) as i32;
			let mut fTemp98: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (1.0 - fTemp87)));
			let mut fTemp99: F32 = fTemp98 + -1.499995;
			let mut fTemp100: F32 = F32::floor(fTemp99);
			let mut fTemp101: F32 = fTemp98 - fTemp100;
			let mut fTemp102: F32 = fTemp98 + (-1.0 - fTemp100);
			let mut fTemp103: F32 = fTemp101 * fTemp102;
			let mut fTemp104: F32 = fTemp98 + (-2.0 - fTemp100);
			let mut fTemp105: F32 = fTemp103 * fTemp104;
			let mut fTemp106: F32 = fTemp98 + (-3.0 - fTemp100);
			let mut iTemp107: i32 = (fTemp99) as i32;
			let mut fTemp108: F32 = 0.5 * (0.020833334 * (fTemp95 * fTemp96 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp97, 4))))) & 8191) as usize] + fTemp105 * fTemp106 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp107, 4))))) & 8191) as usize]) + (fTemp88 + (-4.0 - fTemp90)) * (fTemp96 * (fTemp94 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp97)))) & 8191) as usize] * fTemp92 - 0.083333336 * fTemp91 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp97, 1))))) & 8191) as usize]) + 0.125 * fTemp93 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp97, 2))))) & 8191) as usize]) - 0.083333336 * fTemp95 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp97, 3))))) & 8191) as usize]) + (fTemp98 + (-4.0 - fTemp100)) * (fTemp106 * (fTemp104 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp107)))) & 8191) as usize] * fTemp102 - 0.083333336 * fTemp101 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp107, 1))))) & 8191) as usize]) + 0.125 * fTemp103 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp107, 2))))) & 8191) as usize]) - 0.083333336 * fTemp105 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp107, 3))))) & 8191) as usize]));
			*output0 = 0.70710677 * (fTemp0 + (if iSlow1 != 0 {fTemp108} else {(if iSlow2 != 0 {0.25 * (0.020833334 * (fTemp51 + fTemp63 + fTemp74 + fTemp85) + fTemp86)} else {0.33333334 * (0.020833334 * (fTemp12 * fTemp13 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp16, 4))))) & 8191) as usize] + fTemp26 * fTemp27 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp28, 4))))) & 8191) as usize] + fTemp37 * fTemp38 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp39, 4))))) & 8191) as usize]) + (fTemp5 + (-4.0 - fTemp7)) * (fTemp13 * (fTemp11 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp16)))) & 8191) as usize] * fTemp9 - 0.083333336 * fTemp8 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp16, 1))))) & 8191) as usize]) + 0.125 * fTemp10 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp16, 2))))) & 8191) as usize]) - 0.083333336 * fTemp12 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp16, 3))))) & 8191) as usize]) + (fTemp19 + (-4.0 - fTemp21)) * (fTemp27 * (fTemp25 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp28)))) & 8191) as usize] * fTemp23 - 0.083333336 * fTemp22 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp28, 1))))) & 8191) as usize]) + 0.125 * fTemp24 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp28, 2))))) & 8191) as usize]) - 0.083333336 * fTemp26 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp28, 3))))) & 8191) as usize]) + (fTemp30 + (-4.0 - fTemp32)) * (fTemp38 * (fTemp36 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp39)))) & 8191) as usize] * fTemp34 - 0.083333336 * fTemp33 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp39, 1))))) & 8191) as usize]) + 0.125 * fTemp35 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp39, 2))))) & 8191) as usize]) - 0.083333336 * fTemp37 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp39, 3))))) & 8191) as usize]))})}));
			let mut fTemp109: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (1.0 - fTemp4)));
			let mut fTemp110: F32 = fTemp109 + -1.499995;
			let mut fTemp111: F32 = F32::floor(fTemp110);
			let mut fTemp112: F32 = fTemp109 - fTemp111;
			let mut fTemp113: F32 = fTemp109 + (-1.0 - fTemp111);
			let mut fTemp114: F32 = fTemp112 * fTemp113;
			let mut fTemp115: F32 = fTemp109 + (-2.0 - fTemp111);
			let mut fTemp116: F32 = fTemp114 * fTemp115;
			let mut fTemp117: F32 = fTemp109 + (-3.0 - fTemp111);
			let mut iTemp118: i32 = (fTemp110) as i32;
			let mut fTemp119: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (1.0 - fTemp18)));
			let mut fTemp120: F32 = fTemp119 + -1.499995;
			let mut fTemp121: F32 = F32::floor(fTemp120);
			let mut fTemp122: F32 = fTemp119 - fTemp121;
			let mut fTemp123: F32 = fTemp119 + (-1.0 - fTemp121);
			let mut fTemp124: F32 = fTemp122 * fTemp123;
			let mut fTemp125: F32 = fTemp119 + (-2.0 - fTemp121);
			let mut fTemp126: F32 = fTemp124 * fTemp125;
			let mut fTemp127: F32 = fTemp119 + (-3.0 - fTemp121);
			let mut iTemp128: i32 = (fTemp120) as i32;
			let mut fTemp129: F32 = F32::max(2.0, F32::min(8189.0, self.fRec0[0] * (fTemp29 + 1.0)));
			let mut fTemp130: F32 = fTemp129 + -1.499995;
			let mut fTemp131: F32 = F32::floor(fTemp130);
			let mut fTemp132: F32 = fTemp129 - fTemp131;
			let mut fTemp133: F32 = fTemp129 + (-1.0 - fTemp131);
			let mut fTemp134: F32 = fTemp132 * fTemp133;
			let mut fTemp135: F32 = fTemp129 + (-2.0 - fTemp131);
			let mut fTemp136: F32 = fTemp134 * fTemp135;
			let mut fTemp137: F32 = fTemp129 + (-3.0 - fTemp131);
			let mut iTemp138: i32 = (fTemp130) as i32;
			*output1 = 0.70710677 * (fTemp14 + (if iSlow1 != 0 {fTemp108} else {(if iSlow2 != 0 {0.25 * (fTemp86 + 0.020833334 * (fTemp63 + fTemp51 + fTemp74 + fTemp85))} else {0.33333334 * (0.020833334 * (fTemp116 * fTemp117 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp118, 4))))) & 8191) as usize] + fTemp126 * fTemp127 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp128, 4))))) & 8191) as usize] + fTemp136 * fTemp137 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp138, 4))))) & 8191) as usize]) + (fTemp109 + (-4.0 - fTemp111)) * (fTemp117 * (fTemp115 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp118)))) & 8191) as usize] * fTemp113 - 0.083333336 * fTemp112 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp118, 1))))) & 8191) as usize]) + 0.125 * fTemp114 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp118, 2))))) & 8191) as usize]) - 0.083333336 * fTemp116 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp118, 3))))) & 8191) as usize]) + (fTemp119 + (-4.0 - fTemp121)) * (fTemp127 * (fTemp125 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp128)))) & 8191) as usize] * fTemp123 - 0.083333336 * fTemp122 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp128, 1))))) & 8191) as usize]) + 0.125 * fTemp124 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp128, 2))))) & 8191) as usize]) - 0.083333336 * fTemp126 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp128, 3))))) & 8191) as usize]) + (fTemp129 + (-4.0 - fTemp131)) * (fTemp137 * (fTemp135 * (0.020833334 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, iTemp138)))) & 8191) as usize] * fTemp133 - 0.083333336 * fTemp132 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp138, 1))))) & 8191) as usize]) + 0.125 * fTemp134 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp138, 2))))) & 8191) as usize]) - 0.083333336 * fTemp136 * self.fVec2[((i32::wrapping_sub(self.IOTA0, std::cmp::min(8192, std::cmp::max(0, i32::wrapping_add(iTemp138, 3))))) & 8191) as usize]))})}));
			self.iVec0[1] = self.iVec0[0];
			self.fRec0[1] = self.fRec0[0];
			self.fRec2[1] = self.fRec2[0];
			self.IOTA0 = i32::wrapping_add(self.IOTA0, 1);
		}
		
	}

}

impl FaustDsp for ChorusDsp {
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
