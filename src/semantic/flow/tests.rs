//! Test suite for control flow analysis module.

use super::*;
use crate::semantic::ir::{Function, Instruction, ValueId};
use crate::semantic::symbol::SymbolId;
use crate::semantic::types::TypeId;

#[test]
fn test_basic_block_id_display() {
    let bb_id = BasicBlockId(42);
    assert_eq!(bb_id.to_string(), "bb42");
}

#[test]
fn test_basic_block_creation() {
    let bb_id = BasicBlockId(0);
    let mut block = BasicBlock::new(bb_id);

    assert_eq!(block.id, bb_id);
    assert!(block.instructions.is_empty());
    assert!(block.terminator.is_none());
    assert!(block.predecessors.is_empty());
    assert!(block.successors.is_empty());
    assert!(!block.has_terminator());

    // Add an instruction
    let inst = Instruction::Binary {
        op: crate::semantic::ir::BinaryOp::Add,
        left: ValueId(0),
        right: ValueId(1),
    };
    block.add_instruction(inst.clone());
    assert_eq!(block.instructions.len(), 1);
    assert_eq!(block.instructions[0], inst);

    // Set terminator
    let terminator = Instruction::Br { target: BasicBlockId(1) };
    block.set_terminator(terminator.clone());
    assert_eq!(block.terminator, Some(terminator));
    assert!(block.has_terminator());
}

#[test]
fn test_cfg_creation() {
    let mut cfg = ControlFlowGraph::new();

    // Entry and exit blocks are created automatically
    assert_eq!(cfg.block_count(), 2);

    let entry = cfg.entry;
    let exit = cfg.exit;
    assert_ne!(entry, exit);

    // Create a new block
    let bb1 = cfg.create_block();
    assert_eq!(cfg.block_count(), 3);
    assert_eq!(bb1, BasicBlockId(2));

    // Add edges
    cfg.add_edge(entry, bb1);
    cfg.add_edge(bb1, exit);

    let entry_block = cfg.get_block(entry).unwrap();
    assert_eq!(entry_block.successors, vec![bb1]);

    let bb1_block = cfg.get_block(bb1).unwrap();
    assert_eq!(bb1_block.predecessors, vec![entry]);
    assert_eq!(bb1_block.successors, vec![exit]);

    let exit_block = cfg.get_block(exit).unwrap();
    assert_eq!(exit_block.predecessors, vec![bb1]);
}

#[test]
fn test_cfg_edge_duplication() {
    let mut cfg = ControlFlowGraph::new();
    let entry = cfg.entry;
    let exit = cfg.exit;

    // Add the same edge multiple times
    cfg.add_edge(entry, exit);
    cfg.add_edge(entry, exit);
    cfg.add_edge(entry, exit);

    // Edge should only appear once
    let entry_block = cfg.get_block(entry).unwrap();
    assert_eq!(entry_block.successors, vec![exit]);

    let exit_block = cfg.get_block(exit).unwrap();
    assert_eq!(exit_block.predecessors, vec![entry]);
}

#[test]
fn test_dominator_tree_diamond() {
    let mut cfg = ControlFlowGraph::new();
    let entry = cfg.entry;
    let exit = cfg.exit;

    // Diamond-shaped CFG
    let bb1 = cfg.create_block();
    let bb2 = cfg.create_block();
    let bb3 = cfg.create_block();

    cfg.add_edge(entry, bb1);
    cfg.add_edge(entry, bb2);
    cfg.add_edge(bb1, bb3);
    cfg.add_edge(bb2, bb3);
    cfg.add_edge(bb3, exit);

    let dt = DominatorTree::compute(&cfg);

    // Entry dominates everything
    assert!(dt.dominates(entry, entry));
    assert!(dt.dominates(entry, bb1));
    assert!(dt.dominates(entry, bb2));
    assert!(dt.dominates(entry, bb3));
    assert!(dt.dominates(entry, exit));

    // bb1 only dominates itself
    assert!(dt.dominates(bb1, bb1));
    assert!(!dt.dominates(bb1, bb2));
    assert!(!dt.dominates(bb1, bb3));
    assert!(!dt.dominates(bb1, exit));

    // bb2 only dominates itself
    assert!(dt.dominates(bb2, bb2));
    assert!(!dt.dominates(bb2, bb1));
    assert!(!dt.dominates(bb2, bb3));
    assert!(!dt.dominates(bb2, exit));

    // bb3 dominates exit
    assert!(dt.dominates(bb3, bb3));
    assert!(dt.dominates(bb3, exit));
    assert!(!dt.dominates(bb3, bb1));
    assert!(!dt.dominates(bb3, bb2));

    // Exit only dominates itself
    assert!(dt.dominates(exit, exit));

    // Test common dominator
    let common = dt.common_dominator(bb1, bb2);
    assert_eq!(common, Some(entry));
}

#[test]
fn test_dominator_tree_loop() {
    let mut cfg = ControlFlowGraph::new();
    let entry = cfg.entry;
    let exit = cfg.exit;

    // Loop structure
    let header = cfg.create_block();
    let body = cfg.create_block();
    let latch = cfg.create_block();

    cfg.add_edge(entry, header);
    cfg.add_edge(header, body);
    cfg.add_edge(header, exit);
    cfg.add_edge(body, latch);
    cfg.add_edge(latch, header);

    let dt = DominatorTree::compute(&cfg);

    // Entry dominates everything
    assert!(dt.dominates(entry, header));
    assert!(dt.dominates(entry, body));
    assert!(dt.dominates(entry, latch));
    assert!(dt.dominates(entry, exit));

    // Header dominates body and latch
    assert!(dt.dominates(header, body));
    assert!(dt.dominates(header, latch));
    assert!(dt.dominates(header, exit));

    // Body dominates latch
    assert!(dt.dominates(body, latch));
    assert!(!dt.dominates(body, header));
    assert!(!dt.dominates(body, exit));

    // Latch doesn't dominate anyone except itself
    assert!(dt.dominates(latch, latch));
    assert!(!dt.dominates(latch, header));
    assert!(!dt.dominates(latch, exit));

    // Immediate dominators
    assert_eq!(dt.idom(header), Some(entry));
    assert_eq!(dt.idom(body), Some(header));
    assert_eq!(dt.idom(latch), Some(body));
    assert_eq!(dt.idom(exit), Some(header));
}

#[test]
fn test_cfg_builder_creation() {
    let mut func = Function::new(
        SymbolId::new(1),
        "test".to_string(),
        Vec::new(),
        TypeId::new(0)
    );

    let builder = CFGBuilder::new(&mut func);
    // Builder is initialized with entry block as current
    assert_eq!(func.cfg.block_count(), 2); // Entry + Exit
}