//! TypeScript AST node type definitions
//!
//! This module defines the categorized types for TypeScript AST nodes,
//! including statements, expressions, declarations, patterns, types, and literals.

use serde::{Deserialize, Serialize};
use std::fmt;

/// AST node kind variants
///
/// Represents all possible TypeScript AST node types, categorized
/// by their syntactic role in the language.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    // --- Statements ---
    /// Block statement: { stmt1; stmt2; }
    Block {
        statements: Vec<NodeId>,
    },
    /// Expression statement: expr;
    ExpressionStatement {
        expression: NodeId,
    },
    /// If statement: if (cond) then { ... } else { ... }
    If {
        condition: NodeId,
        then_statement: NodeId,
        else_statement: Option<NodeId>,
    },
    /// For statement: for (init; cond; update) body { ... }
    For {
        initializer: Option<NodeId>,
        condition: Option<NodeId>,
        increment: Option<NodeId>,
        body: NodeId,
    },
    /// For-of statement: for (const x of iterable) { ... }
    ForOf {
        variable: NodeId,
        iterable: NodeId,
        body: NodeId,
    },
    /// While statement: while (cond) { ... }
    While {
        condition: NodeId,
        body: NodeId,
    },
    /// Do-while statement: do { ... } while (cond);
    DoWhile {
        body: NodeId,
        condition: NodeId,
    },
    /// Return statement: return expr;
    Return {
        value: Option<NodeId>,
    },
    /// Break statement: break label?;
    Break {
        label: Option<String>,
    },
    /// Continue statement: continue label?;
    Continue {
        label: Option<String>,
    },
    /// Switch statement: switch (expr) { case ... }
    Switch {
        expression: NodeId,
        cases: Vec<SwitchCase>,
    },
    /// Try-catch-finally statement
    Try {
        try_block: NodeId,
        catch_clause: Option<CatchClause>,
        finally_block: Option<NodeId>,
    },
    /// Throw statement: throw expr;
    Throw {
        expression: NodeId,
    },
    /// Variable statement: let x = 1, y = 2;
    VariableStatement {
        declarations: Vec<VariableDeclaration>,
    },

    // --- Expressions ---
    /// Identifier: name
    Identifier {
        name: String,
    },
    /// Literal value (number, string, boolean, etc.)
    Literal(Literal),
    /// Array literal: [1, 2, 3]
    Array {
        elements: Vec<ArrayElement>,
    },
    /// Object literal: { a: 1, b: 2 }
    Object {
        properties: Vec<ObjectProperty>,
    },
    /// Binary expression: a + b
    Binary {
        operator: BinaryOperator,
        left: NodeId,
        right: NodeId,
    },
    /// Unary expression: -x, !x, ++x
    Unary {
        operator: UnaryOperator,
        operand: NodeId,
    },
    /// Assignment: x = 1, x += 2
    Assignment {
        operator: AssignmentOperator,
        target: NodeId,
        value: NodeId,
    },
    /// Conditional (ternary) expression: a ? b : c
    Conditional {
        test: NodeId,
        consequent: NodeId,
        alternate: NodeId,
    },
    /// Function call: func(a, b)
    Call {
        callee: NodeId,
        arguments: Vec<NodeId>,
    },
    /// Member access: obj.prop or obj["prop"]
    Member {
        object: NodeId,
        property: MemberProperty,
    },
    /// New expression: new Class()
    New {
        callee: NodeId,
        arguments: Vec<NodeId>,
    },
    /// Arrow function: (a, b) => { ... }
    ArrowFunction {
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: NodeId,
    },
    /// Function expression: function() { ... }
    FunctionExpression {
        name: Option<String>,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: NodeId,
    },
    /// This expression
    This,
    /// Super expression
    Super,
    /// Template literal: `hello ${name}!`
    Template {
        parts: Vec<TemplatePart>,
    },
    /// Sequence expression: (a, b, c)
    Sequence {
        expressions: Vec<NodeId>,
    },

    // --- Declarations ---
    /// Function declaration: function name() { ... }
    FunctionDeclaration {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: NodeId,
    },
    /// Class declaration: class Name { ... }
    ClassDeclaration {
        name: String,
        extends: Option<NodeId>,
        members: Vec<ClassMember>,
    },
    /// Interface declaration: interface Name { ... }
    InterfaceDeclaration {
        name: String,
        extends: Vec<NodeId>,
        body: Vec<NodeId>,
    },
    /// Type alias declaration: type Name = Type;
    TypeAliasDeclaration {
        name: String,
        type_params: Option<Vec<TypeParameter>>,
        type_annotation: TypeAnnotation,
    },
    /// Enum declaration: enum Name { A, B }
    EnumDeclaration {
        name: String,
        members: Vec<EnumMember>,
    },
    /// Import declaration: import { x } from 'mod'
    ImportDeclaration {
        specifiers: Vec<ImportSpecifier>,
        source: String,
    },
    /// Export declaration: export { x }
    ExportDeclaration {
        specifiers: Vec<ExportSpecifier>,
    },

    // --- Patterns ---
    /// Object pattern: { a, b: c }
    ObjectPattern {
        properties: Vec<PatternProperty>,
    },
    /// Array pattern: [a, b, ...rest]
    ArrayPattern {
        elements: Vec<PatternElement>,
    },
    /// Rest pattern: ...rest
    RestPattern {
        argument: NodeId,
    },

    // --- Types ---
    /// Named type reference: string, number, MyClass
    TypeReference {
        name: String,
        type_params: Option<Vec<TypeAnnotation>>,
    },
    /// Array type: string[]
    ArrayType {
        element_type: Box<TypeAnnotation>,
    },
    /// Union type: A | B | C
    UnionType {
        types: Vec<TypeAnnotation>,
    },
    /// Intersection type: A & B
    IntersectionType {
        types: Vec<TypeAnnotation>,
    },
    /// Tuple type: [string, number]
    TupleType {
        elements: Vec<TypeAnnotation>,
    },
    /// Function type: (a: string, b: number) => void
    FunctionType {
        params: Vec<Parameter>,
        return_type: Box<TypeAnnotation>,
    },
    /// Generic type parameter: <T extends>
    TypeParameter {
        name: String,
        constraint: Option<TypeAnnotation>,
        default: Option<TypeAnnotation>,
    },
    /// Type annotation: : string
    TypeAnnotation {
        type_annotation: Box<TypeAnnotation>,
    },

    // --- Module ---
    /// Source file
    SourceFile {
        statements: Vec<NodeId>,
    },
    /// Module declaration: declare module "name" { ... }
    ModuleDeclaration {
        name: String,
        body: Vec<NodeId>,
    },
}

/// Binary operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    /// Arithmetic: +
    Add,
    /// Arithmetic: -
    Subtract,
    /// Arithmetic: *
    Multiply,
    /// Arithmetic: /
    Divide,
    /// Arithmetic: %
    Modulo,
    /// Arithmetic: **
    Exponent,
    /// Bitwise: &
    BitwiseAnd,
    /// Bitwise: |
    BitwiseOr,
    /// Bitwise: ^
    BitwiseXor,
    /// Bitwise: <<
    LeftShift,
    /// Bitwise: >>
    RightShift,
    /// Bitwise: >>>
    UnsignedRightShift,
    /// Comparison: ==
    Equal,
    /// Comparison: ===
    StrictEqual,
    /// Comparison: !=
    NotEqual,
    /// Comparison: !==
    NotStrictEqual,
    /// Comparison: <
    LessThan,
    /// Comparison: <=
    LessThanOrEqual,
    /// Comparison: >
    GreaterThan,
    /// Comparison: >=
    GreaterThanOrEqual,
    /// Logical: &&
    LogicalAnd,
    /// Logical: ||
    LogicalOr,
    /// Nullish coalescing: ??
    NullishCoalescing,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Subtract => write!(f, "-"),
            BinaryOperator::Multiply => write!(f, "*"),
            BinaryOperator::Divide => write!(f, "/"),
            BinaryOperator::Modulo => write!(f, "%"),
            BinaryOperator::Exponent => write!(f, "**"),
            BinaryOperator::BitwiseAnd => write!(f, "&"),
            BinaryOperator::BitwiseOr => write!(f, "|"),
            BinaryOperator::BitwiseXor => write!(f, "^"),
            BinaryOperator::LeftShift => write!(f, "<<"),
            BinaryOperator::RightShift => write!(f, ">>"),
            BinaryOperator::UnsignedRightShift => write!(f, ">>>"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::StrictEqual => write!(f, "==="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::NotStrictEqual => write!(f, "!=="),
            BinaryOperator::LessThan => write!(f, "<"),
            BinaryOperator::LessThanOrEqual => write!(f, "<="),
            BinaryOperator::GreaterThan => write!(f, ">"),
            BinaryOperator::GreaterThanOrEqual => write!(f, ">="),
            BinaryOperator::LogicalAnd => write!(f, "&&"),
            BinaryOperator::LogicalOr => write!(f, "||"),
            BinaryOperator::NullishCoalescing => write!(f, "??"),
        }
    }
}

/// Unary operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Arithmetic: -x
    Minus,
    /// Arithmetic: +x
    Plus,
    /// Logical: !x
    LogicalNot,
    /// Bitwise: ~x
    BitwiseNot,
    /// Prefix increment: ++x
    IncrementPrefix,
    /// Postfix increment: x++
    IncrementPostfix,
    /// Prefix decrement: --x
    DecrementPrefix,
    /// Postfix decrement: x--
    DecrementPostfix,
    /// Typeof: typeof x
    Typeof,
    /// Void: void x
    Void,
    /// Delete: delete x
    Delete,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Minus => write!(f, "-"),
            UnaryOperator::Plus => write!(f, "+"),
            UnaryOperator::LogicalNot => write!(f, "!"),
            UnaryOperator::BitwiseNot => write!(f, "~"),
            UnaryOperator::IncrementPrefix => write!(f, "++"),
            UnaryOperator::IncrementPostfix => write!(f, "++"),
            UnaryOperator::DecrementPrefix => write!(f, "--"),
            UnaryOperator::DecrementPostfix => write!(f, "--"),
            UnaryOperator::Typeof => write!(f, "typeof"),
            UnaryOperator::Void => write!(f, "void"),
            UnaryOperator::Delete => write!(f, "delete"),
        }
    }
}

/// Assignment operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssignmentOperator {
    /// Simple assignment: =
    Assign,
    /// Addition: +=
    AddAssign,
    /// Subtraction: -=
    SubtractAssign,
    /// Multiplication: *=
    MultiplyAssign,
    /// Division: /=
    DivideAssign,
    /// Modulo: %=
    ModuloAssign,
    /// Left shift: <<=
    LeftShiftAssign,
    /// Right shift: >>=
    RightShiftAssign,
    /// Unsigned right shift: >>>=
    UnsignedRightShiftAssign,
    /// Bitwise AND: &=
    BitwiseAndAssign,
    /// Bitwise OR: |=
    BitwiseOrAssign,
    /// Bitwise XOR: ^=
    BitwiseXorAssign,
    /// Exponent: **=
    ExponentAssign,
}

impl fmt::Display for AssignmentOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssignmentOperator::Assign => write!(f, "="),
            AssignmentOperator::AddAssign => write!(f, "+="),
            AssignmentOperator::SubtractAssign => write!(f, "-="),
            AssignmentOperator::MultiplyAssign => write!(f, "*="),
            AssignmentOperator::DivideAssign => write!(f, "/="),
            AssignmentOperator::ModuloAssign => write!(f, "%="),
            AssignmentOperator::LeftShiftAssign => write!(f, "<<="),
            AssignmentOperator::RightShiftAssign => write!(f, ">>="),
            AssignmentOperator::UnsignedRightShiftAssign => write!(f, ">>>="),
            AssignmentOperator::BitwiseAndAssign => write!(f, "&="),
            AssignmentOperator::BitwiseOrAssign => write!(f, "|="),
            AssignmentOperator::BitwiseXorAssign => write!(f, "^="),
            AssignmentOperator::ExponentAssign => write!(f, "**="),
        }
    }
}

/// Literal values
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// String literal
    String(String),
    /// Number literal
    Number(f64),
    /// Boolean literal
    Boolean(bool),
    /// Null literal
    Null,
    /// Undefined literal
    Undefined,
    /// BigInt literal
    BigInt(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Null => write!(f, "null"),
            Literal::Undefined => write!(f, "undefined"),
            Literal::BigInt(n) => write!(f, "{}n", n),
        }
    }
}

/// Node identifier
///
/// Unique identifier for a node in the AST arena.
/// Used to refer to nodes without storing pointers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId {
    index: u32,
}

impl NodeId {
    /// Create a new node ID
    pub fn new(index: u32) -> Self {
        Self { index }
    }

    /// Get the index value
    pub fn index(self) -> u32 {
        self.index
    }
}

impl From<u32> for NodeId {
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

/// Function parameter
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<NodeId>,
    pub is_rest: bool,
}

/// Variable declaration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: String,
    pub kind: VariableKind,
    pub initializer: Option<NodeId>,
    pub type_annotation: Option<TypeAnnotation>,
}

/// Variable declaration kind
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableKind {
    /// let
    Let,
    /// const
    Const,
    /// var
    Var,
}

/// Type parameter declaration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeParameter {
    pub name: String,
    pub constraint: Option<TypeAnnotation>,
    pub default: Option<TypeAnnotation>,
}

/// Type annotation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    /// Simple type reference
    TypeReference {
        name: String,
        type_params: Option<Vec<TypeAnnotation>>,
    },
    /// Array type
    ArrayType(Box<TypeAnnotation>),
    /// Union type
    UnionType(Vec<TypeAnnotation>),
    /// Function type
    FunctionType {
        params: Vec<Parameter>,
        return_type: Box<TypeAnnotation>,
    },
    /// Unknown type
    Unknown,
}

/// Array element (can be expression or spread)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ArrayElement {
    /// Regular element
    Element(NodeId),
    /// Spread element: ...expr
    Spread(NodeId),
}

/// Object property
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectProperty {
    pub key: PropertyKey,
    pub value: NodeId,
    pub is_shorthand: bool,
}

/// Property key
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyKey {
    /// Identifier key
    Identifier(String),
    /// String literal key
    String(String),
    /// Number literal key
    Number(f64),
    /// Computed key: [expr]
    Computed(NodeId),
}

/// Member property access
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MemberProperty {
    /// Property access: obj.prop
    Identifier(String),
    /// Computed access: obj[expr]
    Computed(NodeId),
}

/// Switch case
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwitchCase {
    pub test: Option<NodeId>, // None for default case
    pub consequent: Vec<NodeId>,
}

/// Catch clause
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    pub variable: Option<NodeId>,
    pub body: NodeId,
}

/// Class member
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClassMember {
    /// Property declaration
    Property {
        name: String,
        value: Option<NodeId>,
        type_annotation: Option<TypeAnnotation>,
        is_static: bool,
        is_readonly: bool,
    },
    /// Method declaration
    Method {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: NodeId,
        is_static: bool,
    },
    /// Constructor
    Constructor {
        params: Vec<Parameter>,
        body: NodeId,
    },
    /// Get accessor
    Getter {
        name: String,
        return_type: Option<TypeAnnotation>,
        body: NodeId,
    },
    /// Set accessor
    Setter {
        name: String,
        params: Vec<Parameter>,
        body: NodeId,
    },
}

/// Import specifier
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportSpecifier {
    /// Named import: import { x } from 'mod'
    Named { name: String, alias: Option<String> },
    /// Namespace import: import * as x from 'mod'
    Namespace(String),
    /// Default import: import x from 'mod'
    Default(String),
}

/// Export specifier
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ExportSpecifier {
    /// Named export: export { x }
    Named { name: String, alias: Option<String> },
    /// Default export: export default value
    Default(NodeId),
}

/// Enum member
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<Literal>,
}

/// Template literal part
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemplatePart {
    /// Static string part
    Static(String),
    /// Expression part: ${expr}
    Expression(NodeId),
}

/// Pattern property
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PatternProperty {
    pub key: PropertyKey,
    pub pattern: NodeId,
    pub is_shorthand: bool,
}

/// Pattern element
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PatternElement {
    /// Regular pattern
    Pattern(Option<NodeId>),
    /// Rest pattern: ...rest
    Rest(NodeId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.index(), 42);

        let id2 = NodeId::from(100);
        assert_eq!(id2.index(), 100);
    }

    #[test]
    fn test_literal_display() {
        assert_eq!(format!("{}", Literal::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", Literal::Number(42.0)), "42");
        assert_eq!(format!("{}", Literal::Boolean(true)), "true");
        assert_eq!(format!("{}", Literal::Null), "null");
    }

    #[test]
    fn test_binary_operator_display() {
        assert_eq!(format!("{}", BinaryOperator::Add), "+");
        assert_eq!(format!("{}", BinaryOperator::StrictEqual), "===");
        assert_eq!(format!("{}", BinaryOperator::LogicalAnd), "&&");
    }

    #[test]
    fn test_unary_operator_display() {
        assert_eq!(format!("{}", UnaryOperator::Minus), "-");
        assert_eq!(format!("{}", UnaryOperator::LogicalNot), "!");
        assert_eq!(format!("{}", UnaryOperator::Typeof), "typeof");
    }

    #[test]
    fn test_assignment_operator_display() {
        assert_eq!(format!("{}", AssignmentOperator::Assign), "=");
        assert_eq!(format!("{}", AssignmentOperator::AddAssign), "+=");
        assert_eq!(format!("{}", AssignmentOperator::MultiplyAssign), "*=");
    }

    #[test]
    fn test_type_annotation() {
        let simple_type = TypeAnnotation::TypeReference {
            name: "string".to_string(),
            type_params: None,
        };
        assert!(matches!(simple_type, TypeAnnotation::TypeReference { .. }));

        let array_type = TypeAnnotation::ArrayType(Box::new(TypeAnnotation::TypeReference {
            name: "number".to_string(),
            type_params: None,
        }));
        assert!(matches!(array_type, TypeAnnotation::ArrayType(_)));
    }

    #[test]
    fn test_parameter() {
        let param = Parameter {
            name: "x".to_string(),
            type_annotation: Some(TypeAnnotation::TypeReference {
                name: "number".to_string(),
                type_params: None,
            }),
            default_value: None,
            is_rest: false,
        };
        assert_eq!(param.name, "x");
        assert!(!param.is_rest);
    }

    #[test]
    fn test_variable_declaration() {
        let decl = VariableDeclaration {
            name: "x".to_string(),
            kind: VariableKind::Const,
            initializer: None,
            type_annotation: None,
        };
        assert_eq!(decl.name, "x");
        assert_eq!(decl.kind, VariableKind::Const);
    }
}
