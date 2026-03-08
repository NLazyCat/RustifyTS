//! CFG builder implementation.
//!
//! Implements a visitor that constructs a control flow graph from an AST.

use crate::parser::ast::AstNode;
use crate::parser::ast::visitor::Visitor;
use crate::semantic::flow::{BasicBlockId, ControlFlowGraph};
use crate::semantic::ir::{Function, Instruction, ValueId};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Control Flow Graph builder.
///
/// This visitor traverses an AST function body and constructs a control flow
/// graph with basic blocks and proper terminator instructions.
#[derive(Debug)]
pub struct CFGBuilder<'a> {
    /// The function being built
    function: &'a mut Function,
    /// Current basic block being written to
    current_block: BasicBlockId,
    /// Stack of loop headers for break/continue handling
    loop_stack: Vec<LoopContext>,
    /// Map from variable names to their current SSA value
    variable_map: FxHashMap<String, ValueId>,
    /// Next available value ID
    next_value_id: u32,
}

/// Context for a loop structure, used to handle break and continue statements.
#[derive(Debug, Clone, Copy)]
struct LoopContext {
    /// Header block of the loop
    header: BasicBlockId,
    /// Exit block of the loop (where break jumps to)
    exit: BasicBlockId,
    /// Continue block of the loop (where continue jumps to)
    continue_block: BasicBlockId,
}

impl<'a> CFGBuilder<'a> {
    /// Create a new CFG builder for the given function.
    pub fn new(function: &'a mut Function) -> Self {
        let entry_block = function.entry_block();

        Self {
            function,
            current_block: entry_block,
            loop_stack: Vec::new(),
            variable_map: FxHashMap::default(),
            next_value_id: 0,
        }
    }

    /// Create a new unique value ID.
    fn create_value(&mut self) -> ValueId {
        let id = ValueId(self.next_value_id);
        self.next_value_id += 1;
        id
    }

    /// Create a new basic block and return its ID.
    fn create_block(&mut self) -> BasicBlockId {
        self.function.create_block()
    }

    /// Set the current block to append instructions to.
    fn set_current_block(&mut self, block_id: BasicBlockId) {
        self.current_block = block_id;
    }

    /// Add an instruction to the current basic block.
    fn add_instruction(&mut self, inst: Instruction) -> ValueId {
        let result = self.create_value();
        self.function.add_instruction(self.current_block, inst);
        result
    }

    /// Set the terminator instruction for the current basic block.
    fn set_terminator(&mut self, terminator: Instruction) {
        self.function.set_terminator(self.current_block, terminator);
    }

    /// Add a control flow edge between two blocks.
    fn add_edge(&mut self, from: BasicBlockId, to: BasicBlockId) {
        self.function.add_edge(from, to);
    }

    /// Push a new loop context onto the stack.
    fn push_loop(&mut self, header: BasicBlockId, exit: BasicBlockId, continue_block: BasicBlockId) {
        self.loop_stack.push(LoopContext {
            header,
            exit,
            continue_block,
        });
    }

    /// Pop the top loop context from the stack.
    fn pop_loop(&mut self) -> Option<LoopContext> {
        self.loop_stack.pop()
    }

    /// Get the current loop context, if any.
    fn current_loop(&self) -> Option<&LoopContext> {
        self.loop_stack.last()
    }

    /// Build the CFG by traversing the given AST node (function body).
    pub fn build(&mut self, node: &'a AstNode<'a>) {
        self.visit_node(node);

        // Ensure the exit block has a return terminator
        let exit_block = self.function.exit_block();
        if self.function.cfg.get_block(exit_block).unwrap().terminator.is_none() {
            self.set_current_block(exit_block);
            self.set_terminator(Instruction::Ret { value: None });
        }
    }
}

impl<'a> Visitor<'a> for CFGBuilder<'a> {
    fn visit_block(&mut self, node: &'a AstNode<'a>) {
        // Visit all statements in the block
        self.default_visit_node(node);
    }

    fn visit_if(&mut self, node: &'a AstNode<'a>) {
        // If statement structure:
        // condition -> then_block -> merge_block
        //            -> else_block -> merge_block

        // Evaluate condition
        let cond_val = self.create_value(); // Would be result of visiting condition expression

        // Create blocks
        let then_block = self.create_block();
        let else_block = self.create_block();
        let merge_block = self.create_block();

        // Add conditional branch from current block
        self.set_terminator(Instruction::CondBr {
            condition: cond_val,
            true_target: then_block,
            false_target: else_block,
        });
        self.add_edge(self.current_block, then_block);
        self.add_edge(self.current_block, else_block);

        // Visit then block
        self.set_current_block(then_block);
        if let Some(then_node) = node.children().get(1) {
            self.visit_node(then_node);
        }
        // Add branch to merge block if not already terminated
        if self.function.cfg.get_block(then_block).unwrap().terminator.is_none() {
            self.set_terminator(Instruction::Br { target: merge_block });
            self.add_edge(then_block, merge_block);
        }

        // Visit else block (if present)
        self.set_current_block(else_block);
        if node.children().len() > 2 {
            if let Some(else_node) = node.children().get(2) {
                self.visit_node(else_node);
            }
        }
        // Add branch to merge block if not already terminated
        if self.function.cfg.get_block(else_block).unwrap().terminator.is_none() {
            self.set_terminator(Instruction::Br { target: merge_block });
            self.add_edge(else_block, merge_block);
        }

        // Continue in merge block
        self.set_current_block(merge_block);
    }

    fn visit_while(&mut self, node: &'a AstNode<'a>) {
        // While loop structure:
        // pre_header -> header -> body -> latch -> header
        //                                    -> exit

        // Create blocks
        let pre_header = self.current_block;
        let header = self.create_block();
        let body = self.create_block();
        let latch = self.create_block();
        let exit = self.create_block();

        // Branch from pre-header to header
        self.set_terminator(Instruction::Br { target: header });
        self.add_edge(pre_header, header);

        // Push loop context
        self.push_loop(header, exit, latch);

        // Process header (condition)
        self.set_current_block(header);
        let cond_val = self.create_value(); // Result of condition expression
        self.set_terminator(Instruction::CondBr {
            condition: cond_val,
            true_target: body,
            false_target: exit,
        });
        self.add_edge(header, body);
        self.add_edge(header, exit);

        // Process body
        self.set_current_block(body);
        if let Some(body_node) = node.children().get(1) {
            self.visit_node(body_node);
        }
        // Branch to latch if not terminated
        if self.function.cfg.get_block(body).unwrap().terminator.is_none() {
            self.set_terminator(Instruction::Br { target: latch });
            self.add_edge(body, latch);
        }

        // Process latch (back to header)
        self.set_current_block(latch);
        self.set_terminator(Instruction::Br { target: header });
        self.add_edge(latch, header);

        // Pop loop context
        self.pop_loop();

        // Continue in exit block
        self.set_current_block(exit);
    }

    fn visit_for(&mut self, node: &'a AstNode<'a>) {
        // For loop structure:
        // init -> pre_header -> header -> body -> latch -> header
        //                                              -> exit

        // Process init expression
        if let Some(init_node) = node.children().get(0) {
            self.visit_node(init_node);
        }

        // Create blocks
        let pre_header = self.current_block;
        let header = self.create_block();
        let body = self.create_block();
        let latch = self.create_block();
        let exit = self.create_block();

        // Branch from pre-header to header
        self.set_terminator(Instruction::Br { target: header });
        self.add_edge(pre_header, header);

        // Push loop context
        self.push_loop(header, exit, latch);

        // Process header (condition)
        self.set_current_block(header);
        let cond_val = self.create_value(); // Result of condition expression
        self.set_terminator(Instruction::CondBr {
            condition: cond_val,
            true_target: body,
            false_target: exit,
        });
        self.add_edge(header, body);
        self.add_edge(header, exit);

        // Process body
        self.set_current_block(body);
        if let Some(body_node) = node.children().get(2) {
            self.visit_node(body_node);
        }
        // Branch to latch if not terminated
        if self.function.cfg.get_block(body).unwrap().terminator.is_none() {
            self.set_terminator(Instruction::Br { target: latch });
            self.add_edge(body, latch);
        }

        // Process update expression
        self.set_current_block(latch);
        if let Some(update_node) = node.children().get(1) {
            self.visit_node(update_node);
        }
        // Branch back to header
        self.set_terminator(Instruction::Br { target: header });
        self.add_edge(latch, header);

        // Pop loop context
        self.pop_loop();

        // Continue in exit block
        self.set_current_block(exit);
    }

    fn visit_return(&mut self, _node: &'a AstNode<'a>) {
        // Return statement - add Ret terminator
        let ret_val = None; // Would be result of visiting return value expression
        self.set_terminator(Instruction::Ret { value: ret_val });

        // Add edge to exit block
        let exit_block = self.function.exit_block();
        self.add_edge(self.current_block, exit_block);

        // Create a new dead block for any subsequent code
        let dead_block = self.create_block();
        self.set_current_block(dead_block);
    }

    fn visit_break(&mut self, _node: &'a AstNode<'a>) {
        // Break statement - jump to loop exit
        let loop_exit = self.current_loop().map(|ctx| ctx.exit);
        if let Some(target) = loop_exit {
            self.set_terminator(Instruction::Br { target });
            self.add_edge(self.current_block, target);

            // Create a new dead block for any subsequent code
            let dead_block = self.create_block();
            self.set_current_block(dead_block);
        }
    }

    fn visit_continue(&mut self, _node: &'a AstNode<'a>) {
        // Continue statement - jump to loop continue block
        let continue_block = self.current_loop().map(|ctx| ctx.continue_block);
        if let Some(target) = continue_block {
            self.set_terminator(Instruction::Br { target });
            self.add_edge(self.current_block, target);

            // Create a new dead block for any subsequent code
            let dead_block = self.create_block();
            self.set_current_block(dead_block);
        }
    }

    fn visit_expression_statement(&mut self, node: &'a AstNode<'a>) {
        // Visit the expression and ignore the result
        self.default_visit_node(node);
    }

    fn visit_variable_statement(&mut self, node: &'a AstNode<'a>) {
        // Visit variable declarations
        self.default_visit_node(node);
    }

    // Default implementation for all other node types
    fn visit_node(&mut self, node: &'a AstNode<'a>) {
        self.default_visit_node(node);
    }
}
