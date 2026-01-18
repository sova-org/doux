mod chorus;
mod coarse;
mod comb;
mod crush;
mod distort;
mod flanger;
mod lag;
mod phaser;

pub use chorus::Chorus;
pub use coarse::Coarse;
pub use comb::Comb;
pub use crush::crush;
pub use distort::{distort, fold, wrap};
pub use flanger::Flanger;
pub use lag::Lag;
pub use phaser::Phaser;
