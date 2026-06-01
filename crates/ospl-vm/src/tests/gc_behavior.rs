use ospl_common::{
    ast::frame::RuntimeFrame,
    inst::{list::List, RuntimeFunction, RuntimeValue},
};

use crate::{arena::MEMMAX, VM};

fn allocate_until_full(vm: &mut VM, label: &str) {
    for i in 0..MEMMAX {
        vm.push_literal(RuntimeValue::Str(format!("{label}-{i}")));
    }
}

#[test]
fn gc_baseline_keeps_stack_values_readable_without_pressure() {
    let mut vm = VM::new();

    vm.push_literal(RuntimeValue::Int(42));
    vm.push_literal(RuntimeValue::Str("still here".to_string()));
    vm.push_literal(RuntimeValue::Bool(true));

    assert_eq!(vm.get_value_top(0), &RuntimeValue::Int(42));
    assert_eq!(
        vm.get_value_top(1),
        &RuntimeValue::Str("still here".to_string()),
    );
    assert_eq!(vm.get_value_top(2), &RuntimeValue::Bool(true));
}

#[test]
fn gc_baseline_keeps_nested_heap_references_readable_without_pressure() {
    let mut vm = VM::new();

    let list_child = vm.push_literal(RuntimeValue::Str("list child".to_string()));
    let function_child = vm.push_literal(RuntimeValue::Str("function child".to_string()));
    let scope_child = vm.push_literal(RuntimeValue::Str("scope child".to_string()));

    let list = vm.push_literal(RuntimeValue::List(List {
        items: vec![list_child],
    }));
    let function = vm.push_literal(RuntimeValue::Function(RuntimeFunction {
        lexical_indexes: vec![function_child],
        code: Vec::new(),
    }));
    let scope = vm.push_literal(RuntimeValue::Scope(RuntimeFrame::new(vec![scope_child])));

    vm.top_mut().indexes.clear();
    vm.top_mut().indexes.extend([list, function, scope]);

    let RuntimeValue::List(list) = vm.get_value_top(0) else {
        panic!("expected list root");
    };
    assert_eq!(
        vm.arena.get(list.items[0]),
        &RuntimeValue::Str("list child".to_string()),
    );

    let RuntimeValue::Function(function) = vm.get_value_top(1) else {
        panic!("expected function root");
    };
    assert_eq!(
        vm.arena.get(function.lexical_indexes[0]),
        &RuntimeValue::Str("function child".to_string()),
    );

    let RuntimeValue::Scope(scope) = vm.get_value_top(2) else {
        panic!("expected scope root");
    };
    assert_eq!(
        vm.arena.get(scope.indexes[0]),
        &RuntimeValue::Str("scope child".to_string()),
    );
}

#[test]
fn gc_baseline_keeps_reachable_cycles_readable_without_pressure() {
    let mut vm = VM::new();

    let a = vm.push_literal(RuntimeValue::List(List::default()));
    let b = vm.push_literal(RuntimeValue::List(List::default()));

    let RuntimeValue::List(list) = vm.arena.get_mut(a) else {
        panic!("expected first cycle node to be a list");
    };
    list.items.push(b);

    let RuntimeValue::List(list) = vm.arena.get_mut(b) else {
        panic!("expected second cycle node to be a list");
    };
    list.items.push(a);

    vm.top_mut().indexes.clear();
    vm.top_mut().indexes.push(a);

    let RuntimeValue::List(a_list) = vm.get_value_top(0) else {
        panic!("expected rooted cycle node");
    };
    let b = a_list.items[0];

    let RuntimeValue::List(b_list) = vm.arena.get(b) else {
        panic!("expected second cycle node");
    };
    assert_eq!(b_list.items[0], a);
}

#[test]
#[ignore = "pending tracing GC: allocation pressure should collect unreachable slots"]
fn gc_keeps_stack_roots_alive_under_allocation_pressure() {
    let mut vm = VM::new();
    let root = vm.push_literal(RuntimeValue::Str("root survives".to_string()));

    allocate_until_full(&mut vm, "garbage");

    vm.top_mut().indexes.clear();
    vm.top_mut().indexes.push(root);

    allocate_until_full(&mut vm, "after-gc");

    assert_eq!(
        vm.get_value_top(0),
        &RuntimeValue::Str("root survives".to_string()),
    );
}

#[test]
#[ignore = "pending tracing GC: collector must trace references stored inside heap values"]
fn gc_traces_references_inside_lists_functions_and_scopes() {
    let mut vm = VM::new();

    let list_child = vm.push_literal(RuntimeValue::Str("list child".to_string()));
    let function_child = vm.push_literal(RuntimeValue::Str("function child".to_string()));
    let scope_child = vm.push_literal(RuntimeValue::Str("scope child".to_string()));

    let list = vm.push_literal(RuntimeValue::List(List {
        items: vec![list_child],
    }));
    let function = vm.push_literal(RuntimeValue::Function(RuntimeFunction {
        lexical_indexes: vec![function_child],
        code: Vec::new(),
    }));
    let scope = vm.push_literal(RuntimeValue::Scope(RuntimeFrame::new(vec![scope_child])));

    vm.top_mut().indexes.clear();
    vm.top_mut().indexes.extend([list, function, scope]);

    allocate_until_full(&mut vm, "pressure");

    let RuntimeValue::List(list) = vm.get_value_top(0) else {
        panic!("expected list root to survive GC");
    };
    assert_eq!(
        vm.arena.get(list.items[0]),
        &RuntimeValue::Str("list child".to_string()),
    );

    let RuntimeValue::Function(function) = vm.get_value_top(1) else {
        panic!("expected function root to survive GC");
    };
    assert_eq!(
        vm.arena.get(function.lexical_indexes[0]),
        &RuntimeValue::Str("function child".to_string()),
    );

    let RuntimeValue::Scope(scope) = vm.get_value_top(2) else {
        panic!("expected scope root to survive GC");
    };
    assert_eq!(
        vm.arena.get(scope.indexes[0]),
        &RuntimeValue::Str("scope child".to_string()),
    );
}

#[test]
#[ignore = "pending tracing GC: unreachable cycles should be collectable"]
fn gc_reclaims_unreachable_cycles_under_allocation_pressure() {
    let mut vm = VM::new();

    let a = vm.push_literal(RuntimeValue::List(List::default()));
    let b = vm.push_literal(RuntimeValue::List(List::default()));

    let RuntimeValue::List(list) = vm.arena.get_mut(a) else {
        panic!("expected first cycle node to be a list");
    };
    list.items.push(b);

    let RuntimeValue::List(list) = vm.arena.get_mut(b) else {
        panic!("expected second cycle node to be a list");
    };
    list.items.push(a);

    vm.top_mut().indexes.clear();

    allocate_until_full(&mut vm, "after-cycle");
}
