pub mod cpuinfo;
mod pit;

pub use pit::init_pit;
pub use pit::read_pit;