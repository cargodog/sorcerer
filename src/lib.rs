pub mod config;
pub mod sorcerer;
pub use sorcerer::*;

// Re-export the protobuf types for testing
pub use sorcerer::spells;
