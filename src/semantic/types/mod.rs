//! Type system module.
//!
//! Implements TypeScript type checking, inference, and type validation.
//! Handles type relationships, compatibility checks, and type operations.

pub mod representation;
pub mod interner;
pub mod unify;
pub mod resolver;
pub mod variance;

pub use representation::*;
pub use interner::*;
pub use unify::*;
pub use resolver::*;
pub use variance::*;

#[cfg(test)]
mod tests;
