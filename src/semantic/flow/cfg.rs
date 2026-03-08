//! Control Flow Graph (CFG) data structures.
//!
//! Defines the basic block and CFG structures that represent control flow
//! in functions.

use crate::semantic::ir::Instruction;
use std::fmt;

/// Unique identifier for a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasicBlockId(pub u32);

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// A basic block is a sequence of instructions with a single entry point
/// and a single exit point (terminator instruction).
#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    /// Unique identifier for this basic block
    pub id: BasicBlockId,
    /// List of instructions in this block (excluding terminator)
    pub instructions: Vec<Instruction>,
    /// Terminator instruction that ends this block
    pub terminator: Option<Instruction>,
    /// Predecessor basic blocks
    pub predecessors: Vec<BasicBlockId>,
    /// Successor basic blocks
    pub successors: Vec<BasicBlockId>,
}

impl BasicBlock {
    /// Create a new empty basic block with the given ID.
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            terminator: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    /// Add an instruction to this basic block.
    pub fn add_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    /// Set the terminator instruction for this basic block.
    pub fn set_terminator(&mut self, terminator: Instruction) {
        debug_assert!(self.terminator.is_none(), "Basic block already has a terminator");
        self.terminator = Some(terminator);
    }

    /// Check if this basic block has a terminator instruction.
    pub fn has_terminator(&self) -> bool {
        self.terminator.is_some()
    }
}

/// Control Flow Graph representing the control flow of a function.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowGraph {
    /// List of basic blocks in this CFG
    pub blocks: Vec<BasicBlock>,
    /// Entry block ID
    pub entry: BasicBlockId,
    /// Exit block ID
    pub exit: BasicBlockId,
    /// Next available basic block ID
    next_block_id: u32,
}

impl ControlFlowGraph {
    /// Create a new empty CFG.
    pub fn new() -> Self {
        let mut cfg = Self {
            blocks: Vec::new(),
            entry: BasicBlockId(0),
            exit: BasicBlockId(0),
            next_block_id: 0,
        };

        // Create entry and exit blocks
        let entry_id = cfg.create_block();
        let exit_id = cfg.create_block();
        cfg.entry = entry_id;
        cfg.exit = exit_id;

        cfg
    }

    /// Create a new basic block and return its ID.
    pub fn create_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.next_block_id);
        self.next_block_id += 1;
        self.blocks.push(BasicBlock::new(id));
        id
    }

    /// Get a reference to a basic block by ID.
    pub fn get_block(&self, id: BasicBlockId) -> Option<&BasicBlock> {
        self.blocks.get(id.0 as usize)
    }

    /// Get a mutable reference to a basic block by ID.
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(id.0 as usize)
    }

    /// Add an edge between two basic blocks.
    pub fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        if let Some(from_block) = self.get_block_mut(from) {
            if !from_block.successors.contains(&to) {
                from_block.successors.push(to);
            }
        }
        if let Some(to_block) = self.get_block_mut(to) {
            if !to_block.predecessors.contains(&from) {
                to_block.predecessors.push(from);
            }
        }
    }

    /// Get the number of basic blocks in this CFG.
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}
