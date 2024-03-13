/// Welcome to the IT Department!
///
/// Our IT Department has a directory of PCs that it maintains and is
/// responsible for. Each PC has fixed hardware settings and a state. The state
/// is mutable, but only under maintenance. Further, every PC has one potential
/// owner. If two PCs have the same owner, the corresponding structure is
/// deduplicated.
///
/// +------------+              +----+         +--------+
/// | Direcotory | 0..* ------> | PC | ------> | Person |
/// +------------+              +----+         +--------+
pub mod pc_directory;
pub mod person;
pub mod pc;