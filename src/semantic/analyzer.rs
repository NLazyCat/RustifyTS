//! Main semantic analyzer module.
//!
//! Orchestrates the entire semantic analysis process, coordinating
//! all sub-components (scope, symbol, types, flow, IR) to produce
//! a fully analyzed program representation.

#[cfg(test)]
mod tests;