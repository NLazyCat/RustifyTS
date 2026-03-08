# Phase 06: 代码生成层

## 目标

实现代码渲染，将 Rust AST 转换为 Rust 源代码。

## 实施计划

### 任务清单

- [ ] 创建 `src/codegen/mod.rs` - 模块接口
- [ ] 创建 `src/codegen/renderer.rs` - 代码渲染器
- [ ] 创建 `src/codegen/formatter.rs` - 代码格式化
- [ ] 创建 `src/codegen/optimizer.rs` - AST 优化
- [ ] 创建 `src/codegen/config.rs` - 渲染配置

### 关键设计

#### 渲染配置

```rust
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// 缩进字符串（如 "  " 或 "    "）
    pub indent: String,

    /// 每行最大长度
    pub max_line_length: usize,

    /// 是否使用字段简写语法 `{ field }` vs `{ field: field }`
    pub use_shorthand: bool,

    /// 是否使用匹配守卫
    pub use_match_guards: bool,

    /// 是否使用 if let / while let
    pub use_let_chains: bool,

    /// 是否使用 trait 别名
    pub use_trait_aliases: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            indent: "    ".to_string(),
            max_line_length: 100,
            use_shorthand: true,
            use_match_guards: true,
            use_let_chains: true,
            use_trait_aliases: true,
        }
    }
}
```

#### 渲染器

```rust
use crate::ast::*;
use anyhow::Result;

pub struct Renderer {
    config: RenderConfig,
    indent_level: usize,
}

impl Renderer {
    pub fn new(config: RenderConfig) -> Self {
        Self {
            config,
            indent_level: 0,
        }
    }

    pub fn render_file(&mut self, file: &RustFile) -> Result<String> {
        let mut code = String::new();

        // 渲染 shebang
        if let Some(ref shebang) = file.shebang {
            code.push_str(shebang);
            code.push('\n');
        }

        // 渲染属性
        for attr in &file.attributes {
            code.push_str(&self.render_attribute(attr)?);
            code.push('\n');
        }

        // 渲染顶层项
        for item in &file.items {
            code.push_str(&self.render_item(item)?);
            code.push_str("\n\n");
        }

        Ok(code)
    }

    pub fn render_item(&mut self, item: &Item) -> Result<String> {
        match item {
            Item::Function(func) => self.render_function(func),
            Item::Struct(struct_) => self.render_struct(struct_),
            Item::Enum(enum_) => self.render_enum(enum_),
            Item::Impl(impl_) => self.render_impl(impl_),
            Item::Use(use_) => self.render_use(use_),
            Item::TypeAlias(alias) => self.render_type_alias(alias),
            Item::Constant(const_) => self.render_constant(const_),
            Item::Static(static_) => self.render_static(static_),
            Item::Mod(module) => self.render_mod(module),
            Item::Union(union) => self.render_union(union),
            Item::Trait(trait_) => self.render_trait(trait_),
            Item::ExternFunction(extern_fn) => self.render_extern_function(extern_fn),
            Item::Macro(macro_) => self.render_macro(macro_),
        }
    }

    pub fn render_function(&mut self, func: &Function) -> Result<String> {
        let mut code = String::new();

        // 渲染属性
        for attr in &func.attributes {
            code.push_str(&self.render_attribute(attr)?);
            code.push('\n');
            self.add_indent(&mut code);
        }

        // 渲染可见性
        code.push_str(&self.render_visibility(&func.visibility));
        code.push(' ');

        // 渲染 unsafe
        if func.is_unsafe {
            code.push_str("unsafe ");
        }

        // 渲染 async
        if func.is_async {
            code.push_str("async ");
        }

        // 渲染 abi
        if let Some(ref abi) = func.abi {
            code.push_str(&format!("extern \"{}\" ", abi));
        }

        // 渲染 fn 关键字
        code.push_str("fn ");

        // 渲染函数名
        code.push_str(&func.name);

        // 渲染泛型参数
        if !func.generics.params.is_empty() {
            code.push_str(&self.render_generics(&func.generics)?);
        }

        // 渲染参数
        code.push('(');
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&self.render_parameter(param)?);
        }
        code.push(')');

        // 渲染返回类型
        if !matches!(func.return_type, Type::Primitive(PrimitiveType::Void)) {
            code.push_str(" -> ");
            code.push_str(&self.render_type(&func.return_type)?);
        }

        // 渲染函数体
        if let Some(ref body) = func.body {
            code.push(' ');
            code.push_str(&self.render_block(body)?);
        } else {
            code.push(';');
        }

        Ok(code)
    }

    pub fn render_struct(&mut self, struct_: &Struct) -> Result<String> {
        let mut code = String::new();

        // 渲染属性
        for attr in &struct_.attributes {
            code.push_str(&self.render_attribute(attr)?);
            code.push('\n');
            self.add_indent(&mut code);
        }

        // 渲染可见性
        code.push_str(&self.render_visibility(&struct_.visibility));
        code.push_str(" struct ");

        // 渲染结构体名
        code.push_str(&struct_.name);

        // 渲染泛型参数
        if !struct_.generics.params.is_empty() {
            code.push_str(&self.render_generics(&struct_.generics)?);
        }

        // 渲染结构体内容
        match &struct_.kind {
            StructKind::Unit => {
                code.push(';');
            }
            StructKind::Tuple(types) => {
                code.push('(');
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_type(ty)?);
                }
                code.push_str(");");
            }
            StructKind::Named(fields) => {
                code.push_str(" {\n");
                self.indent_level += 1;

                for field in fields {
                    self.add_indent(&mut code);
                    code.push_str(&self.render_field(field)?);
                    code.push_str(",\n");
                }

                self.indent_level -= 1;
                self.add_indent(&mut code);
                code.push('}');
            }
        }

        Ok(code)
    }

    pub fn render_enum(&mut self, enum_: &Enum) -> Result<String> {
        let mut code = String::new();

        // 渲染属性
        for attr in &enum_.attributes {
            code.push_str(&self.render_attribute(attr)?);
            code.push('\n');
            self.add_indent(&mut code);
        }

        // 渲染可见性
        code.push_str(&self.render_visibility(&enum_.visibility));
        code.push_str(" enum ");

        // 渲染枚举名
        code.push_str(&enum_.name);

        // 渲染泛型参数
        if !enum_.generics.params.is_empty() {
            code.push_str(&self.render_generics(&enum_.generics)?);
        }

        // 渲染枚举内容
        code.push_str(" {\n");
        self.indent_level += 1;

        for variant in &enum_.variants {
            self.add_indent(&mut code);
            code.push_str(&self.render_variant(variant)?);
            code.push_str(",\n");
        }

        self.indent_level -= 1;
        self.add_indent(&mut code);
        code.push('}');

        Ok(code)
    }

    pub fn render_type(&mut self, ty: &Type) -> Result<String> {
        match ty {
            Type::Primitive(prim) => Ok(self.render_primitive_type(prim)),
            Type::Ref { mutable, lifetime, inner } => {
                let mut code = String::new();
                code.push('&');
                if let Some(lt) = lifetime {
                    code.push_str(lt);
                    code.push(' ');
                }
                if *mutable {
                    code.push_str("mut ");
                }
                code.push_str(&self.render_type(inner)?);
                Ok(code)
            }
            Type::Slice(inner) => {
                let mut code = String::new();
                code.push('[');
                code.push_str(&self.render_type(inner)?);
                code.push(']');
                Ok(code)
            }
            Type::Array(inner, size) => {
                Ok(format!("[{}; {}]", self.render_type(inner)?, size))
            }
            Type::Tuple(types) => {
                let mut code = String::new();
                code.push('(');
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_type(ty)?);
                }
                code.push(')');
                Ok(code)
            }
            Type::Path { path, generics } => {
                if generics.is_empty() {
                    Ok(path.clone())
                } else {
                    let mut code = path.clone();
                    code.push('<');
                    for (i, ty) in generics.iter().enumerate() {
                        if i > 0 {
                            code.push_str(", ");
                        }
                        code.push_str(&self.render_type(ty)?);
                    }
                    code.push('>');
                    Ok(code)
                }
            }
            Type::Never => Ok("!".to_string()),
            Type::Inferred => Ok("_".to_string()),
            Type::RawPtr { mutable, inner } => {
                let mut code = String::new();
                code.push('*');
                if *mutable {
                    code.push_str("mut ");
                } else {
                    code.push_str("const ");
                }
                code.push_str(&self.render_type(inner)?);
                Ok(code)
            }
            Type::FunctionPointer { params, return_type, is_unsafe } => {
                let mut code = String::new();
                if *is_unsafe {
                    code.push_str("unsafe ");
                }
                code.push_str("fn(");
                for (i, ty) in params.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_type(ty)?);
                }
                code.push_str(") -> ");
                code.push_str(&self.render_type(return_type)?);
                Ok(code)
            }
            Type::Closure { params, return_type, kind } => {
                let mut code = match kind {
                    ClosureKind::Fn => "fn",
                    ClosureKind::FnMut => "fn mut",
                    ClosureKind::FnOnce => "fn once",
                };
                code.to_string() // 简化实现
            }
        }
    }

    pub fn render_expr(&mut self, expr: &Expr) -> Result<String> {
        match expr {
            Expr::Literal(lit) => Ok(self.render_literal(lit)),
            Expr::Path(path) => self.render_path_expr(path),
            Expr::MethodCall { object, method, args } => {
                let mut code = String::new();

                // 处理对象
                let obj_str = self.render_expr(object)?;
                if needs_parens_for_call(object) {
                    code.push('(');
                    code.push_str(&obj_str);
                    code.push(')');
                } else {
                    code.push_str(&obj_str);
                }

                code.push('.');
                code.push_str(method);

                code.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_expr(arg)?);
                }
                code.push(')');

                Ok(code)
            }
            Expr::FieldAccess { object, field } => {
                let obj_str = self.render_expr(object)?;
                if needs_parens_for_field_access(object) {
                    Ok(format!("({}).{}", obj_str, field))
                } else {
                    Ok(format!("{}.{}", obj_str, field))
                }
            }
            Expr::Index { object, index } => {
                let obj_str = self.render_expr(object)?;
                let idx_str = self.render_expr(index)?;
                Ok(format!("{}[{}]", obj_str, idx_str))
            }
            Expr::Unary { op, operand } => {
                let op_str = self.render_unary_op(*op);
                let operand_str = self.render_expr(operand)?;
                if needs_parens_for_unary(operand) {
                    Ok(format!("{}({})", op_str, operand_str))
                } else {
                    Ok(format!("{}{}", op_str, operand_str))
                }
            }
            Expr::Binary { op, left, right } => {
                let left_str = self.render_expr(left)?;
                let right_str = self.render_expr(right)?;
                let op_str = self.render_binary_op(*op);

                let if_left_needs_parens = needs_parens_for_binary_left(*op, left);
                let if_right_needs_parens = needs_parens_for_binary_right(*op, right);

                let left_paren = if if_left_needs_parens { "(" } else { "" };
                let right_paren = if if_right_needs_parens { "(" } else { "" };
                let right_end_paren = if if_right_needs_parens { ")" } else { "" };
                let left_end_paren = if if_left_needs_parens { ")" } else { "" };

                Ok(format!("{}{}{} {} {}{}{}", left_paren, left_str, left_end_paren, op_str, right_paren, right_str, right_end_paren))
            }
            Expr::Assign { left, right } => {
                Ok(format!("{} = {}", self.render_expr(left)?, self.render_expr(right)?))
            }
            Expr::AssignOp { op, left, right } => {
                let op_str = self.render_binary_op(*(*op).add_assign_version());
                Ok(format!("{} {} {}", self.render_expr(left)?, op_str, self.render_expr(right)?))
            }
            Expr::ChainAssign { left, right } => {
                Ok(format!("{} = {}", self.render_expr(left)?, self.render_expr(right)?))
            }
            Expr::Call { func, args } => {
                let mut code = String::new();
                let func_str = self.render_expr(func)?;

                if needs_parens_for_call(func) {
                    code.push('(');
                    code.push_str(&func_str);
                    code.push(')');
                } else {
                    code.push_str(&func_str);
                }

                code.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_expr(arg)?);
                }
                code.push(')');

                Ok(code)
            }
            Expr::Closure { params, body, move_ } => {
                let mut code = String::new();
                if *move_ {
                    code.push_str("move ");
                }
                code.push('|');
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    // 简化：只渲染参数名
                    if let Pattern::Binding { name, .. } = &param.pattern {
                        code.push_str(name);
                    }
                }
                code.push_str("| ");
                code.push_str(&self.render_expr(body)?);
                Ok(code)
            }
            Expr::Block(block) => self.render_block(block),
            Expr::If { condition, then_branch, else_branch } => {
                let mut code = String::new();
                code.push_str("if ");
                code.push_str(&self.render_expr(condition)?);
                code.push(' ');
                code.push_str(&self.render_block(then_branch)?);
                if let Some(else_expr) = else_branch {
                    code.push_str(" else ");
                    if matches!(else_expr.as_ref(), Expr::If { .. }) {
                        // 嵌套的 if，不加大括号
                        code.push_str(&self.render_expr(else_expr)?);
                    } else {
                        code.push_str(&self.render_block(else_expr.as_block().unwrap())?);
                    }
                }
                Ok(code)
            }
            Expr::While { condition, body } => {
                let mut code = String::new();
                code.push_str("while ");
                code.push_str(&self.render_expr(condition)?);
                code.push(' ');
                code.push_str(&self.render_block(body)?);
                Ok(code)
            }
            Expr::For { pattern, iter, body } => {
                let mut code = String::new();
                code.push_str("for ");
                code.push_str(&self.render_pattern(pattern)?);
                code.push_str(" in ");
                code.push_str(&self.render_expr(iter)?);
                code.push(' ');
                code.push_str(&self.render_block(body)?);
                Ok(code)
            }
            Expr::Loop(body) => {
                Ok(format!("loop {}", self.render_block(body)?))
            }
            Expr::Match { expr, arms } => {
                let mut code = String::new();
                code.push_str("match ");
                code.push_str(&self.render_expr(expr)?);
                code.push_str(" {\n");
                self.indent_level += 1;

                for arm in arms {
                    self.add_indent(&mut code);
                    code.push_str(&self.render_match_arm(arm)?);
                    code.push_str(",\n");
                }

                self.indent_level -= 1;
                self.add_indent(&mut code);
                code.push('}');

                Ok(code)
            }
            Expr::Break(value) => {
                if let Some(v) = value {
                    Ok(format!("break {}", self.render_expr(v)?))
                } else {
                    Ok("break".to_string())
                }
            }
            Expr::Continue(label) => {
                if let Some(l) = label {
                    Ok(format!("continue {}", l))
                } else {
                    Ok("continue".to_string())
                }
            }
            Expr::Return(value) => {
                if let Some(v) = value {
                    Ok(format!("return {}", self.render_expr(v)?))
                } else {
                    Ok("return".to_string())
                }
            }
            Expr::Array(elements) => {
                let mut code = String::new();
                code.push('[');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_expr(elem)?);
                }
                code.push(']');
                Ok(code)
            }
            Expr::RepeatArray { value, count } => {
                Ok(format!("[{}; {}]", self.render_expr(value)?, self.render_expr(count)?))
            }
            Expr::Tuple(elements) => {
                let mut code = String::new();
                code.push('(');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(&self.render_expr(elem)?);
                }
                code.push(')');
                Ok(code)
            }
            Expr::Struct { path, fields, base } => {
                let mut code = String::new();
                code.push_str(path);
                code.push_str(" {\n");
                self.indent_level += 1;

                for field in fields {
                    self.add_indent(&mut code);
                    code.push_str(&field.name);
                    code.push_str(": ");
                    code.push_str(&self.render_expr(&field.value)?);
                    code.push_str(",\n");
                }

                if let Some(base_expr) = base {
                    self.add_indent(&mut code);
                    code.push_str("..");
                    code.push_str(&self.render_expr(base_expr)?);
                    code.push('\n');
                }

                self.indent_level -= 1;
                self.add_indent(&mut code);
                code.push('}');

                Ok(code)
            }
            Expr::StructShorthand { path, fields } => {
                let mut code = String::new();
                code.push_str(path);
                code.push_str(" { ");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        code.push_str(", ");
                    }
                    code.push_str(field);
                }
                code.push_str(" }");
                Ok(code)
            }
            Expr::Range { start, end, inclusive } => {
                let sep = if *inclusive { "..=" } else { ".." };
                let start_str = start.as_ref()
                    .map(|e| self.render_expr(e).unwrap())
                    .unwrap_or_default();
                let end_str = end.as_ref()
                    .map(|e| self.render_expr(e).unwrap())
                    .unwrap_or_default();
                Ok(format!("{}{}{}", start_str, sep, end_str))
            }
            Expr::Async(block) => {
                Ok(format!("async {}", self.render_block(block)?))
            }
            Expr::Await(expr) => {
                Ok(format!("{}.await()", self.render_expr(expr)?))
            }
            Expr::Unsafe(block) => {
                Ok(format!("unsafe {}", self.render_block(block)?))
            }
            Expr::Cast { expr, target_type } => {
                Ok(format!("{} as {}", self.render_expr(expr)?, self.render_type(target_type)?))
            }
            Expr::Paren(expr) => {
                Ok(format!("({})", self.render_expr(expr)?))
            }
            Expr::TypeAnnotation { expr, type_annotation } => {
                Ok(format!("{}: {}", self.render_expr(expr)?, self.render_type(type_annotation)?))
            }
        }
    }

    pub fn render_block(&mut self, block: &Block) -> Result<String> {
        let mut code = String::new();
        code.push_str("{\n");
        self.indent_level += 1;

        for stmt in &block.statements {
            self.add_indent(&mut code);
            code.push_str(&self.render_statement(stmt)?);
            code.push('\n');
        }

        if let Some(ref tail) = block.tail {
            self.add_indent(&mut code);
            code.push_str(&self.render_expr(tail)?);
        }

        self.indent_level -= 1;
        code.push('\n');
        self.add_indent(&mut code);
        code.push('}');

        Ok(code)
    }

    pub fn render_statement(&mut self, stmt: &Statement) -> Result<String> {
        match stmt {
            Statement::Let { pattern, type_annotation, init } => {
                let mut code = String::new();
                code.push_str("let ");
                code.push_str(&self.render_pattern(pattern)?);

                if let Some(ty) = type_annotation {
                    code.push_str(": ");
                    code.push_str(&self.render_type(ty)?);
                }

                if let Some(init_expr) = init {
                    code.push_str(" = ");
                    code.push_str(&self.render_expr(init_expr)?);
                }

                code.push(';');
                Ok(code)
            }
            Statement::Expr(expr) => {
                Ok(format!("{};", self.render_expr(expr)?))
            }
            Statement::Semicolon => Ok(";".to_string()),
            Statement::Item(item) => self.render_item(item),
        }
    }

    // 辅助方法
    fn add_indent(&self, code: &mut String) {
        for _ in 0..self.indent_level {
            code.push_str(&self.config.indent);
        }
    }

    fn render_visibility(&self, vis: &Visibility) -> String {
        match vis {
            Visibility::Private => String::new(),
            Visibility::Public => "pub".to_string(),
            Visibility::Crate => "pub(crate) ".to_string(),
            Visibility::Super => "pub(super) ".to_string(),
            Visibility::Restricted(path) => format!("pub({}) ", path),
        }
    }

    fn render_primitive_type(&self, prim: PrimitiveType) -> String {
        match prim {
            PrimitiveType::Bool => "bool".to_string(),
            PrimitiveType::Char => "char".to_string(),
            PrimitiveType::Str => "str".to_string(),
            PrimitiveType::I8 => "i8".to_string(),
            PrimitiveType::I16 => "i16".to_string(),
            PrimitiveType::I32 => "i32".to_string(),
            PrimitiveType::I64 => "i64".to_string(),
            PrimitiveType::I128 => "i128".to_string(),
            PrimitiveType::ISize => "isize".to_string(),
            PrimitiveType::U8 => "u8".to_string(),
            PrimitiveType::U16 => "u16".to_string(),
            PrimitiveType::U32 => "u32".to_string(),
            PrimitiveType::U64 => "u64".to_string(),
            PrimitiveType::U128 => "u128".to_string(),
            PrimitiveType::USize => "usize".to_string(),
            PrimitiveType::F32 => "f32".to_string(),
            PrimitiveType::F64 => "f64".to_string(),
        }
    }

    fn render_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::Bool(b) => b.to_string(),
            Literal::Char(c) => format!("'{}'", c),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Int(i) => i.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Byte(b) => format!("b'{}'", b),
        }
    }

    fn render_unary_op(&self, op: UnaryOp) -> &'static str {
        match op {
            UnaryOp::Negate => "-",
            UnaryOp::Not => "!",
            UnaryOp::Deref => "*",
            UnaryOp::Ref => "&",
            UnaryOp::RefMut => "&mut ",
        }
    }

    fn render_binary_op(&self, op: BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::AddAssign => "+=",
            BinaryOp::SubAssign => "-=",
            BinaryOp::MulAssign => "*=",
            BinaryOp::DivAssign => "/=",
            BinaryOp::RemAssign => "%=",
            BinaryOp::BitAndAssign => "&=",
            BinaryOp::BitOrAssign => "|=",
            BinaryOp::BitXorAssign => "^=",
            BinaryOp::ShlAssign => "<<=",
            BinaryOp::ShrAssign => ">>=",
        }
    }
}

// 辅助函数用于确定是否需要加括号
fn needs_parens_for_call(expr: &Expr) -> bool {
    match expr {
        Expr::Binary { .. } | Expr::Unary { .. } => true,
        _ => false,
    }
}

fn needs_parens_for_field_access(expr: &Expr) -> bool {
    match expr {
        Expr::Binary { .. } | Expr::Unary { .. } => true,
        _ => false,
    }
}

fn needs_parens_for_unary(expr: &Expr) -> bool {
    matches!(expr, Expr::Binary { .. } | Expr::Unary { .. })
}

fn needs_parens_for_binary_left(op: BinaryOp, left: &Expr) -> bool {
    match (op, left) {
        // 对于右结合运算符，左边的相同优先级不需要括号
        (BinaryOp::Assign, Expr::Assign { .. }) => true,
        (BinaryOp::Assign, Expr::AssignOp { .. }) => true,
        _ => false,
    }
}

fn needs_parens_for_binary_right(op: BinaryOp, right: &Expr) -> bool {
    match (op, right) {
        // 赋值运算符右边通常需要括号，除非是另一个赋值
        (BinaryOp::Assign, Expr::Binary { op: right_op, .. })
            if *right_op != BinaryOp::Assign => true,
        _ => false,
    }
}
```

### 测试计划

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_function() {
        let renderer = Renderer::new(RenderConfig::default());

        let func = Function {
            attributes: vec![],
            visibility: Visibility::Private,
            is_unsafe: false,
            is_async: false,
            abi: None,
            name: "add".to_string(),
            generics: Generics { params: vec![], where_clauses: vec![] },
            params: vec![
                Parameter {
                    attributes: vec![],
                    is_mut: false,
                    pattern: Pattern::Binding {
                        name: "a".to_string(),
                        is_mut: false,
                        is_ref: false,
                        subpattern: None,
                    },
                    type_annotation: Type::Primitive(PrimitiveType::I32),
                },
                Parameter {
                    attributes: vec![],
                    is_mut: false,
                    pattern: Pattern::Binding {
                        name: "b".to_string(),
                        is_mut: false,
                        is_ref: false,
                        subpattern: None,
                    },
                    type_annotation: Type::Primitive(PrimitiveType::I32),
                },
            ],
            return_type: Type::Primitive(PrimitiveType::I32),
            body: Some(Block {
                statements: vec![],
                tail: Some(Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Path(PathExpr {
                        segments: vec![PathSegment {
                            name: "a".to_string(),
                            generics: vec![],
                        }],
                    })),
                    right: Box::new(Expr::Path(PathExpr {
                        segments: vec![PathSegment {
                            name: "b".to_string(),
                            generics: vec![],
                        }],
                    })),
                })),
            }),
        };

        let code = renderer.render_function(&func).unwrap();
        assert!(code.contains("fn add(a: i32, b: i32) -> i32"));
        assert!(code.contains("a + b"));
    }
}
```

## 验收标准

- [ ] `src/codegen/` 模块编译通过
- [ ] 可以渲染所有 Rust 顶层项
- [ ] 可以渲染所有 Rust 表达式
- [ ] 可以渲染所有 Rust 语句
- [ ] 可以渲染所有 Rust 类型
- [ ] 生成的代码可以编译
- [ ] 单元测试通过

## 依赖

Phase 05 必须完成

## 估计时间

3 天

## 风险

1. **运算符优先级** - 运算符优先级处理可能不正确
   - 缓解：编写完整的测试用例

2. **格式化问题** - 生成的代码格式可能不符合 rustfmt
   - 缓解：使用 rustfmt 后处理

## 下一步

完成后进入 Phase 07: 集成测试
