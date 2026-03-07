//! Control flow analysis module.
//!
//! Builds and analyzes control flow graphs, performs reachability analysis,
//! and tracks variable initialization and usage throughout the program.

pub mod cfg;
pub mod builder;
pub mod dominance;

pub use cfg::*;
pub use builder::*;
pub use dominance::*;

#[cfg(test)]
mod tests;