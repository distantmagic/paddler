mod lmdb;
mod memory;

pub use self::lmdb::Lmdb;
pub use self::memory::Memory;

pub trait FleetManagementDatabase {}
