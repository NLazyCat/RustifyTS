//! Test suite for Intermediate Representation (IR) module.

use super::*;
use crate::semantic::flow::BasicBlockId;
use crate::semantic::symbol::SymbolId;
use crate::semantic::types::TypeId;

#[test]
fn test_value_id_display() {
    let val_id = ValueId(42);
    assert_eq!(val_id.to_string(), "%42");
}

#[test]
fn test_function_creation() {
    let func_id = SymbolId::new(1);
    let name = "test_function".to_string();
    let params = vec![
        ("param1".to_string(), TypeId::new(2)),
        ("param2".to_string(), TypeId::new(3)),
    ];
    let return_type = TypeId::new(4);

    let func = Function::new(func_id, name.clone(), params.clone(), return_type);

    assert_eq!(func.id, func_id);
    assert_eq!(func.name, name);
    assert_eq!(func.params, params);
    assert_eq!(func.return_type, return_type);

    // Check that entry and exit blocks exist
    let entry_block = func.entry_block();
    let exit_block = func.exit_block();
    assert_ne!(entry_block, exit_block);
    assert_eq!(func.cfg.block_count(), 2); // Entry + Exit
}

#[test]
fn test_function_value_creation() {
    let mut func = Function::new(
        SymbolId::new(1),
        "test".to_string(),
        Vec::new(),
        TypeId::new(0)
    );

    let val1 = func.create_value();
    let val2 = func.create_value();
    let val3 = func.create_value();

    assert_eq!(val1, ValueId(0));
    assert_eq!(val2, ValueId(1));
    assert_eq!(val3, ValueId(2));
}

#[test]
fn test_function_block_management() {
    let mut func = Function::new(
        SymbolId::new(1),
        "test".to_string(),
        Vec::new(),
        TypeId::new(0)
    );

    let entry_block = func.entry_block();
    let new_block = func.create_block();

    assert_eq!(func.cfg.block_count(), 3); // Entry + Exit + new_block

    // Add an instruction to the new block
    let inst = Instruction::Binary {
        op: BinaryOp::Add,
        left: ValueId(0),
        right: ValueId(1),
    };
    func.add_instruction(new_block, inst.clone());

    let block = func.cfg.get_block(new_block).unwrap();
    assert_eq!(block.instructions.len(), 1);
    assert_eq!(block.instructions[0], inst);
    assert!(block.terminator.is_none());

    // Set terminator
    let terminator = Instruction::Br { target: entry_block };
    func.set_terminator(new_block, terminator.clone());

    let block = func.cfg.get_block(new_block).unwrap();
    assert_eq!(block.terminator, Some(terminator));
}

#[test]
fn test_module_creation() {
    let module_name = "test_module".to_string();
    let mut module = SemanticModule::new(module_name.clone());

    assert_eq!(module.name, module_name);
    assert!(module.functions.is_empty());

    // Add a function
    let func = Function::new(
        SymbolId::new(1),
        "func1".to_string(),
        Vec::new(),
        TypeId::new(0)
    );
    module.add_function(func);

    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].name, "func1");

    // Look up function by ID
    let found = module.get_function(SymbolId::new(1));
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "func1");

    let not_found = module.get_function(SymbolId::new(999));
    assert!(not_found.is_none());
}

#[test]
fn test_instruction_variants() {
    // Test Binary instruction
    let bin_op = Instruction::Binary {
        op: BinaryOp::Add,
        left: ValueId(0),
        right: ValueId(1),
    };
    assert!(matches!(bin_op, Instruction::Binary { op: BinaryOp::Add, .. }));

    // Test Unary instruction
    let un_op = Instruction::Unary {
        op: UnaryOp::Neg,
        operand: ValueId(0),
    };
    assert!(matches!(un_op, Instruction::Unary { op: UnaryOp::Neg, .. }));

    // Test Load/Store
    let load = Instruction::Load { address: ValueId(0) };
    assert!(matches!(load, Instruction::Load { .. }));

    let store = Instruction::Store {
        address: ValueId(0),
        value: ValueId(1),
    };
    assert!(matches!(store, Instruction::Store { .. }));

    // Test Call
    let call = Instruction::Call {
        function: ValueId(0),
        args: vec![ValueId(1), ValueId(2)],
    };
    assert!(matches!(call, Instruction::Call { args, .. } if args.len() == 2));

    // Test Branch instructions
    let br = Instruction::Br { target: BasicBlockId(0) };
    assert!(matches!(br, Instruction::Br { .. }));

    let cond_br = Instruction::CondBr {
        condition: ValueId(0),
        true_target: BasicBlockId(1),
        false_target: BasicBlockId(2),
    };
    assert!(matches!(cond_br, Instruction::CondBr { .. }));

    // Test Return
    let ret_void = Instruction::Ret { value: None };
    assert!(matches!(ret_void, Instruction::Ret { value: None }));

    let ret_val = Instruction::Ret { value: Some(ValueId(0)) };
    assert!(matches!(ret_val, Instruction::Ret { value: Some(_) }));

    // Test Phi
    let phi = Instruction::Phi {
        incoming: vec![(ValueId(0), BasicBlockId(0)), (ValueId(1), BasicBlockId(1))],
    };
    assert!(matches!(phi, Instruction::Phi { incoming, .. } if incoming.len() == 2));

    // Test Alloca
    let alloca = Instruction::Alloca { ty: TypeId::new(0) };
    assert!(matches!(alloca, Instruction::Alloca { .. }));

    // Test GEP
    let gep = Instruction::GetElementPtr {
        base: ValueId(0),
        indices: vec![ValueId(1), ValueId(2)],
    };
    assert!(matches!(gep, Instruction::GetElementPtr { indices, .. } if indices.len() == 2));

    // Test Constant
    let int_const = Instruction::Constant {
        ty: TypeId::new(0),
        value: ConstantValue::Int(42),
    };
    assert!(matches!(int_const, Instruction::Constant { value: ConstantValue::Int(42), .. }));

    let float_const = Instruction::Constant {
        ty: TypeId::new(1),
        value: ConstantValue::Float(3.14),
    };
    assert!(matches!(float_const, Instruction::Constant { value: ConstantValue::Float(_), .. }));

    let bool_const = Instruction::Constant {
        ty: TypeId::new(2),
        value: ConstantValue::Bool(true),
    };
    assert!(matches!(bool_const, Instruction::Constant { value: ConstantValue::Bool(true), .. }));
}