//! IR instruction definitions.
//!
//! Defines all instruction variants for the IR.

use crate::semantic::flow::BasicBlockId;
use crate::semantic::types::TypeId;
use std::fmt;

/// Unique identifier for an SSA value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueId(pub u32);

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Binary operation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    /// Integer addition
    Add,
    /// Integer subtraction
    Sub,
    /// Integer multiplication
    Mul,
    /// Integer division
    Div,
    /// Integer remainder
    Rem,
    /// Bitwise AND
    And,
    /// Bitwise OR
    Or,
    /// Bitwise XOR
    Xor,
    /// Left shift
    Shl,
    /// Right shift
    Shr,
    /// Equality comparison
    Eq,
    /// Inequality comparison
    Ne,
    /// Less than comparison
    Lt,
    /// Less than or equal comparison
    Le,
    /// Greater than comparison
    Gt,
    /// Greater than or equal comparison
    Ge,
}

/// Unary operation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// Integer negation
    Neg,
    /// Bitwise NOT
    Not,
    /// Logical NOT
    LNot,
}

/// Represents a single IR instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: ValueId,
        right: ValueId,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        operand: ValueId,
    },

    /// Load a value from memory
    Load {
        address: ValueId,
    },

    /// Store a value to memory
    Store {
        address: ValueId,
        value: ValueId,
    },

    /// Function call
    Call {
        function: ValueId,
        args: Vec<ValueId>,
    },

    /// Unconditional branch to a basic block
    Br {
        target: BasicBlockId,
    },

    /// Conditional branch to one of two basic blocks
    CondBr {
        condition: ValueId,
        true_target: BasicBlockId,
        false_target: BasicBlockId,
    },

    /// Return from a function
    Ret {
        value: Option<ValueId>,
    },

    /// Phi node for SSA construction
    Phi {
        incoming: Vec<(ValueId, BasicBlockId)>,
    },

    /// Stack allocation
    Alloca {
        ty: TypeId,
    },

    /// Get element pointer (array/struct indexing)
    GetElementPtr {
        base: ValueId,
        indices: Vec<ValueId>,
    },

    /// Constant value
    Constant {
        ty: TypeId,
        value: ConstantValue,
    },
}

/// Represents a constant value in the IR.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    /// Integer constant
    Int(i64),
    /// Floating point constant
    Float(f64),
    /// Boolean constant
    Bool(bool),
    /// String constant (index into string table)
    String(u32),
    /// Null pointer
    Null,
    /// Undefined value
    Undef,
}
