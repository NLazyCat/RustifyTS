//! IR function and basic block representation.
//!
//! Defines the function and basic block structures for the IR.

use crate::semantic::flow::{BasicBlock, BasicBlockId, ControlFlowGraph};
use crate::semantic::symbol::SymbolId;
use crate::semantic::types::TypeId;
use crate::semantic::ir::{Instruction, ValueId};
use std::fmt;

/// Represents a function in the IR.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    /// Unique symbol ID for this function
    pub id: SymbolId,
    /// Name of the function
    pub name: String,
    /// Function parameters (name, type)
    pub params: Vec<(String, TypeId)>,
    /// Return type of the function
    pub return_type: TypeId,
    /// Control flow graph for this function
    pub cfg: ControlFlowGraph,
    /// Value ID counter for generating new unique values
    next_value_id: u32,
}

impl Function {
    /// Create a new function with the given ID, name, parameters, and return type.
    pub fn new(id: SymbolId, name: String, params: Vec<(String, TypeId)>, return_type: TypeId) -> Self {
        Self {
            id,
            name,
            params,
            return_type,
            cfg: ControlFlowGraph::new(),
            next_value_id: 0,
        }
    }

    /// Create a new unique value ID.
    pub fn create_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    /// Get the entry block of this function.
    pub fn entry_block(&self) -> BasicBlockId {
        self.cfg.entry
    }

    /// Get the exit block of this function.
    pub fn exit_block(&self) -> BasicBlockId {
        self.cfg.exit
    }

    /// Create a new basic block in this function's CFG.
    pub fn create_block(&mut self) -> BasicBlockId {
        self.cfg.create_block()
    }

    /// Add an instruction to the end of a basic block.
    pub fn add_instruction(&mut self, block_id: BasicBlockId, inst: Instruction) {
        if let Some(block) = self.cfg.get_block_mut(block_id) {
            block.add_instruction(inst);
        }
    }

    /// Set the terminator instruction for a basic block.
    pub fn set_terminator(&mut self, block_id: BasicBlockId, terminator: Instruction) {
        if let Some(block) = self.cfg.get_block_mut(block_id) {
            block.set_terminator(terminator);
        }
    }

    /// Add a control flow edge between two blocks.
    pub fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        self.cfg.add_edge(from, to);
    }
}
