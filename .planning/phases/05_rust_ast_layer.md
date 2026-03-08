# Phase 05: Rust AST 层

## 目标

实现 Rust 语义表示，独立于代码生成。

## 实施计划

### 任务清单

- [ ] 创建 `src/ast/mod.rs` - 模块接口
- [ ] 创建 `src/ast/types.rs` - Rust 类型
- [ ] 创建 `src/ast/items.rs` - 顶层项
- [ ] 创建 `src/ast/expr.rs` - 表达式
- [ ] 创建 `src/ast/stmt.rs` - 语句
- [ ] 创建 `src/ast/pattern.rs` - 模式
- [ ] 创建 `src/ast/attributes.rs` - 属性
- [ ] 创建 `src/ast/visibility.rs` - 可见性

### 关键设计

#### Rust 类型系统

```rust
/// Rust 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// 基本类型
    Primitive(PrimitiveType),

    /// 引用类型 &T 或 &mut T
    Ref {
        mutable: bool,
        lifetime: Option<String>,
        inner: Box<Type>,
    },

    /// 原始指针 *const T 或 *mut T
    RawPtr {
        mutable: bool,
        inner: Box<Type>,
    },

    /// 切片 [T]
    Slice(Box<Type>),

    /// 数组 [T; N]
    Array(Box<Type>, usize),

    /// 元组
    Tuple(Vec<Type>),

    /// 路径类型（带类型参数）
    Path {
        path: String,
        generics: Vec<Type>,
    },

    /// 函数指针类型
    FunctionPointer {
        params: Vec<Type>,
        return_type: Box<Type>,
        is_unsafe: bool,
    },

    /// 闭包类型
    Closure {
        params: Vec<Type>,
        return_type: Box<Type>,
        kind: ClosureKind,
    },

    /// Never 类型 !
    Never,

    /// 占位类型 _
    Inferred,
}

/// 基本类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    Bool,
    Char,
    Str,

    I8, I16, I32, I64, I128, ISize,
    U8, U16, U32, U64, U128, USize,
    F32, F64,
}

/// 闭包类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosureKind {
    Fn,
    FnMut,
    FnOnce,
}
```

#### 顶层项

```rust
/// Rust 文件
#[derive(Debug, Clone)]
pub struct RustFile {
    pub items: Vec<Item>,
    pub attributes: Vec<Attribute>,
    pub shebang: Option<String>,
}

/// 顶层项
#[derive(Debug, Clone)]
pub enum Item {
    /// 函数
    Function(Function),

    /// 结构体
    Struct(Struct),

    /// 枚举
    Enum(Enum),

    /// 联合体
    Union(Union),

    /// Trait 定义
    Trait(TraitDefinition),

    /// impl 块
    Impl(Impl),

    /// 类型别名
    TypeAlias(TypeAlias),

    /// use 声明
    Use(Use),

    /// mod 声明
    Mod(Module),

    /// 常量
    Constant(Constant),

    /// 静态变量
    Static(Static),

    /// 外部函数
    ExternFunction(ExternFunction),

    /// 宏定义
    Macro(Macro),
}

/// 函数
#[derive(Debug, Clone)]
pub struct Function {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub is_unsafe: bool,
    pub is_async: bool,
    pub abi: Option<String>,
    pub name: String,
    pub generics: Generics,
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub body: Option<Block>,
}

/// 参数
#[derive(Debug, Clone)]
pub struct Parameter {
    pub attributes: Vec<Attribute>,
    pub is_mut: bool,
    pub pattern: Pattern,
    pub type_annotation: Type,
}

/// 结构体
#[derive(Debug, Clone)]
pub struct Struct {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Generics,
    pub kind: StructKind,
}

/// 结构体类型
#[derive(Debug, Clone)]
pub enum StructKind {
    /// 单元结构体 `struct Unit;`
    Unit,

    /// 元组结构体 `struct Tuple(i32, String);`
    Tuple(Vec<Type>),

    /// 普通结构体 `struct Named { field: i32 }`
    Named(Vec<Field>),
}

/// 字段
#[derive(Debug, Clone)]
pub struct Field {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub type_annotation: Type,
}

/// 枚举
#[derive(Debug, Clone)]
pub struct Enum {
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: String,
    pub generics: Generics,
    pub variants: Vec<Variant>,
}

/// 枚举变体
#[derive(Debug, Clone)]
pub struct Variant {
    pub attributes: Vec<Attribute>,
    pub name: String,
    pub kind: VariantKind,
}

/// 变体类型
#[derive(Debug, Clone)]
pub enum VariantKind {
    /// 单元变体 `Variant`
    Unit,

    /// 元组变体 `Variant(i32)`
    Tuple(Vec<Type>),

    /// 结构变体 `Variant { field: i32 }`
    Named(Vec<Field>),
}

/// impl 块
#[derive(Debug, Clone)]
pub struct Impl {
    pub attributes: Vec<Attribute>,
    pub generics: Generics,
    pub for_type: Type,
    pub trait_name: Option<String>,
    pub items: Vec<ImplItem>,
}

/// impl 块中的项
#[derive(Debug, Clone)]
pub enum ImplItem {
    Function(Function),
    Constant(Constant),
    TypeAlias(TypeAlias),
}

/// 泛型参数
#[derive(Debug, Clone)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub where_clauses: Vec<WhereClause>,
}

/// 泛型参数
#[derive(Debug, Clone)]
pub enum GenericParam {
    Type {
        name: String,
        bounds: Vec<TypeBound>,
    },
    Lifetime(String),
    Const {
        name: String,
        type_annotation: Type,
    },
}

/// 类型边界
#[derive(Debug, Clone)]
pub enum TypeBound {
    Trait(String),
    Lifetime(String),
}
```

#### 表达式

```rust
/// Rust 表达式
#[derive(Debug, Clone)]
pub enum Expr {
    /// 字面量
    Literal(Literal),

    /// 路径表达式（变量、函数调用等）
    Path(PathExpr),

    /// 方法调用 `obj.method(args)`
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    /// 字段访问 `obj.field`
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },

    /// 索引 `arr[idx]`
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },

    /// 一元运算符 `!expr`, `-expr`, `*expr`, `&expr`
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },

    /// 二元运算符 `a + b`, `a == b`, etc.
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// 赋值 `x = y`
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// 复合赋值 `x += y`
    AssignOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// 链式赋值 `x = y = z`
    ChainAssign {
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// 函数调用 `func(args)`
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },

    /// 闭包 `|x| x + 1`
    Closure {
        params: Vec<Parameter>,
        body: Box<Expr>,
        move_: bool,
    },

    /// 块 `{ statements }`
    Block(Block),

    /// if 表达式
    If {
        condition: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Box<Expr>>,
    },

    /// if let 表达式
    IfLet {
        pattern: Pattern,
        expr: Box<Expr>,
        then_branch: Block,
        else_branch: Option<Block>,
    },

    /// while 循环
    While {
        condition: Box<Expr>,
        body: Block,
    },

    /// while let 循环
    WhileLet {
        pattern: Pattern,
        expr: Box<Expr>,
        body: Block,
    },

    /// for 循环
    For {
        pattern: Pattern,
        iter: Box<Expr>,
        body: Block,
    },

    /// loop 循环
    Loop(Block),

    /// match 表达式
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// break 表达式
    Break(Option<Box<Expr>>),

    /// continue 表达式
    Continue(Option<String>),

    /// return 表达式
    Return(Option<Box<Expr>>),

    /// 数组 `[1, 2, 3]`
    Array(Vec<Expr>),

    /// 重复数组 `[x; n]`
    RepeatArray {
        value: Box<Expr>,
        count: Box<Expr>,
    },

    /// 元组 `(1, 2, 3)`
    Tuple(Vec<Expr>),

    /// 结构体初始化 `Struct { field: value }`
    Struct {
        path: String,
        fields: Vec<StructField>,
        base: Option<Box<Expr>>,
    },

    /// 结构体初始化简写 `Struct { field }`
    StructShorthand {
        path: String,
        fields: Vec<String>,
    },

    /// 范围 `a..b`, `a..=b`, `..b`, `a..`
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },

    /// async 块
    Async(Block),

    /// await 表达式
    Await(Box<Expr>),

    /// unsafe 块
    Unsafe(Block),

    /// 类型转换 `x as T`
    Cast {
        expr: Box<Expr>,
        target_type: Type,
    },

    /// 括号 `(expr)`
    Paren(Box<Expr>),

    /// 类型标注 `expr: T`
    TypeAnnotation {
        expr: Box<Expr>,
        type_annotation: Type,
    },
}

/// 字面量
#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Char(char),
    String(String),
    Int(i128),
    Float(f64),
    Byte(u8),
}

/// 路径表达式
#[derive(Debug, Clone)]
pub struct PathExpr {
    pub segments: Vec<PathSegment>,
}

/// 路径段
#[derive(Debug, Clone)]
pub struct PathSegment {
    pub name: String,
    pub generics: Vec<Type>,
}

/// 块
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub tail: Option<Box<Expr>>,
}

/// 语句
#[derive(Debug, Clone)]
pub enum Statement {
    /// let 语句
    Let {
        pattern: Pattern,
        type_annotation: Option<Type>,
        init: Option<Expr>,
    },

    /// 表达式语句
    Expr(Expr),

    /// 空语句 `;`
    Semicolon,

    /// Item (函数定义等)
    Item(Item),
}

/// 模式
#[derive(Debug, Clone)]
pub enum Pattern {
    /// 字面量模式
    Literal(Literal),

    /// 路径模式（枚举变体、结构体等）
    Path(PathExpr),

    /// 通配符模式 `_`
    Wildcard,

    /// 变量绑定模式
    Binding {
        name: String,
        is_mut: bool,
        is_ref: bool,
        subpattern: Option<Box<Pattern>>,
    },

    /// or 模式 `a | b`
    Or(Vec<Pattern>),

    /// 元组模式 `(a, b)`
    Tuple(Vec<Pattern>),

    /// 数组模式 `[a, b]`
    Array(Vec<Pattern>),

    /// 范围模式 `a..b`
    Range {
        start: Option<Box<Pattern>>,
        end: Option<Box<Pattern>>,
    },

    /// 结构体模式 `Struct { field }`
    Struct {
        path: String,
        fields: Vec<PatternField>,
        rest: bool,
    },
}

/// 结构体模式字段
#[derive(Debug, Clone)]
pub struct PatternField {
    pub name: String,
    pub pattern: Pattern,
}

/// match 分支
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// 结构体字段
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub value: Expr,
}
```

#### 运算符

```rust
/// 一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,   // -
    Not,      // !
    Deref,    // *
    Ref,      // &
    RefMut,   // &mut
}

/// 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // 算术运算符
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    Rem,       // %
    BitAnd,    // &
    BitOr,     // |
    BitXor,    // ^
    Shl,       // <<
    Shr,       // >>

    // 比较运算符
    Eq,        // ==
    NotEq,     // !=
    Lt,        // <
    Gt,        // >
    LtEq,      // <=
    GtEq,      // >=

    // 逻辑运算符
    And,       // &&
    Or,        // ||

    // 赋值运算符（仅用于 AssignOp）
    AddAssign,      // +=
    SubAssign,      // -=
    MulAssign,      // *=
    DivAssign,      // /=
    RemAssign,      // %=
    BitAndAssign,   // &=
    BitOrAssign,    // |=
    BitXorAssign,   // ^=
    ShlAssign,      // <<=
    ShrAssign,      // >>=
}
```

#### 可见性和属性

```rust
/// 可见性
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
    Crate,
    Super,
    Restricted(String),  // pub(crate::module)
}

/// 属性
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: AttributeArgs,
    pub style: AttributeStyle,
}

/// 属性参数
#[derive(Debug, Clone)]
pub enum AttributeArgs {
    None,
    List(Vec<AttributeArg>),
    KeyValue(String, AttributeArg),
}

/// 属性参数
#[derive(Debug, Clone)]
pub enum AttributeArg {
    Ident(String),
    Path(String),
    Literal(Literal),
    List(Vec<AttributeArg>),
}

/// 属性风格
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeStyle {
    Outer,     // #[attribute]
    Inner,     // #![attribute]
}
```

### 测试计划

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_type() {
        let ty = Type::Primitive(PrimitiveType::I32);
        assert_eq!(ty, Type::Primitive(PrimitiveType::I32));
    }

    #[test]
    fn test_function_item() {
        let func = Function {
            attributes: vec![],
            visibility: Visibility::Private,
            is_unsafe: false,
            is_async: false,
            abi: None,
            name: "main".to_string(),
            generics: Generics { params: vec![], where_clauses: vec![] },
            params: vec![],
            return_type: Type::Primitive(PrimitiveType::Void),
            body: None,
        };

        assert_eq!(func.name, "main");
    }
}
```

## 验收标准

- [ ] `src/ast/` 模块编译通过
- [ ] 所有 Rust AST 类型定义完整
- [ ] 表达式类型覆盖所有 Rust 表达式
- [ ] 语句类型覆盖所有 Rust 语句
- [ ] 类型系统覆盖所有 Rust 类型
- [ ] 单元测试通过

## 依赖

Phase 04 必须完成

## 估计时间

3 天

## 风险

1. **AST 复杂性** - Rust AST 非常复杂
   - 缓解：先实现常用特性，逐步支持完整语法

## 下一步

完成后进入 Phase 06: 代码生成层
