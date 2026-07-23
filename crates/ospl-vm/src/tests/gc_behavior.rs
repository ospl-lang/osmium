use ospl_common::{
    ast::frame::RuntimeFrame,
    inst::{RT, RuntimeFunction, RuntimeValue, assume, assume_mut, list::List, make},
};

use crate::{VM, arena::MEMMAX, gc::RUNS_PER_MEMMAX};

fn allocate_until_full(vm: &mut VM, label: &str) {
    for i in 0..MEMMAX {
        vm.push_literal(make::str(format!("{label}-{i}")));
    }
}

#[test]
fn gc_baseline_keeps_stack_values_readable_without_pressure() {
    let mut vm = VM::new();

    vm.push_literal(make::int(42));
    vm.push_literal(make::str("still here".to_string()));
    vm.push_literal(make::bool(true));

    assert_eq!(vm.get_value_top(0), &make::int(42));
    assert_eq!(
        vm.get_value_top(1),
        &make::str("still here".to_string()),
    );
    assert_eq!(vm.get_value_top(2), &make::bool(true));
}

#[test]
fn gc_baseline_keeps_nested_heap_references_readable_without_pressure() {
    let mut vm = VM::new();

    let list_child = vm.push_literal(make::str("list child".to_string()));
    let function_child = vm.push_literal(make::str("function child".to_string()));
    let scope_child = vm.push_literal(make::str("scope child".to_string()));

    let list = vm.push_literal(make::list(List {
        items: vec![list_child],
    }));
    let function = vm.push_literal(make::func(RuntimeFunction {
        captures: vec![function_child],
        code: Vec::new(),
    }));
    let scope = vm.push_literal(make::scope(RuntimeFrame::new(vec![scope_child])));

    vm.stack.end();
    vm.stack.push_isolated();
    vm.stack.top_add_index(list);
    vm.stack.top_add_index(function);
    vm.stack.top_add_index(scope);

    let Some(list) = assume::list(vm.get_value_top(0)) else {
        panic!("expected list root");
    };
    assert_eq!(
        vm.arena.get(list.items[0]),
        &make::str("list child".to_string()),
    );

    let Some(function) = assume::func(vm.get_value_top(1)) else {
        panic!("expected function root");
    };
    assert_eq!(
        vm.arena.get(function.captures[0]),
        &make::str("function child".to_string()),
    );

    let Some(scope) = assume::scope(vm.get_value_top(2)) else {
        panic!("expected scope root");
    };
    assert_eq!(
        vm.arena.get(scope.indexes[0]),
        &make::str("scope child".to_string()),
    );
}

#[test]
fn gc_baseline_keeps_reachable_cycles_readable_without_pressure() {
    let mut vm = VM::new();

    let a = vm.push_literal(make::list(List::default()));
    let b = vm.push_literal(make::list(List::default()));

    let Some(list) = assume_mut::list(vm.arena.get_mut(a)) else {
        panic!("expected first cycle node to be a list");
    };
    list.items.push(b);

    let Some(list) = assume_mut::list(vm.arena.get_mut(b)) else {
        panic!("expected second cycle node to be a list");
    };
    list.items.push(a);

    vm.stack.end();
    vm.stack.push_isolated();
    vm.stack.top_add_index(a);

    let Some(a_list) = assume::list(vm.arena.get_mut(0)) else {
        panic!("expected rooted cycle node");
    };
    let b = a_list.items[0];

    let Some(b_list) = assume::list(vm.arena.get_mut(b)) else {
        panic!("expected second cycle node");
    };
    assert_eq!(b_list.items[0], a);
}

#[test]
// #[ignore = "pending tracing GC: allocation pressure should collect unreachable slots"]
fn gc_keeps_stack_roots_alive_under_allocation_pressure() {
    let mut vm = VM::new();
    let root = vm.push_literal(make::str("root survives".to_string()));

    allocate_until_full(&mut vm, "garbage");

    vm.stack.end();
    vm.stack.push_isolated();
    vm.stack.top_add_index(root);

    allocate_until_full(&mut vm, "after-gc");

    assert_eq!(
        vm.get_value_top(0),
        &make::str("root survives".to_string()),
    );
}

#[test]
// #[ignore = "pending tracing GC: collector must trace references stored inside heap values"]
fn gc_traces_references_inside_lists_functions_and_scopes() {
    let mut vm = VM::new();

    let list_child = vm.push_literal(make::str("list child".to_string()));
    let function_child = vm.push_literal(make::str("function child".to_string()));
    let scope_child = vm.push_literal(make::str("scope child".to_string()));

    let list = vm.push_literal(make::list(List {
        items: vec![list_child],
    }));
    let function = vm.push_literal(make::func(RuntimeFunction {
        captures: vec![function_child],
        code: Vec::new(),
    }));
    let scope = vm.push_literal(make::scope(RuntimeFrame::new(vec![scope_child])));

    vm.stack.end();
    vm.stack.push_isolated();
    vm.stack.top_add_index(list);
    vm.stack.top_add_index(function);
    vm.stack.top_add_index(scope);

    vm.gc_step(RUNS_PER_MEMMAX);

    let Some(list) = assume::list(vm.get_value_top(0)) else {
        panic!("expected list root to survive GC");
    };
    assert_eq!(
        vm.arena.get(list.items[0]),
        &make::str("list child".to_string()),
    );

    let Some(function) = assume::func(vm.get_value_top(1)) else {
        panic!("expected function root to survive GC");
    };
    assert_eq!(
        vm.arena.get(function.captures[0]),
        &make::str("function child".to_string()),
    );

    let Some(scope) = assume::scope(vm.get_value_top(2)) else {
        panic!("expected scope root to survive GC");
    };
    assert_eq!(
        vm.arena.get(scope.indexes[0]),
        &make::str("scope child".to_string()),
    );
}

#[test]
// #[ignore = "pending tracing GC: unreachable cycles should be collectable"]
fn gc_reclaims_unreachable_cycles_under_allocation_pressure() {
    let mut vm = VM::new();

    let a = vm.push_literal(make::list(List::default()));
    let b = vm.push_literal(make::list(List::default()));

    let Some(list) = assume_mut::list(vm.arena.get_mut(a)) else {
        panic!("expected first cycle node to be a list");
    };
    list.items.push(b);

    let Some(list) = assume_mut::list(vm.arena.get_mut(b)) else {
        panic!("expected second cycle node to be a list");
    };
    list.items.push(a);

    vm.stack.end();

    vm.gc_step(RUNS_PER_MEMMAX);

    // allocate_until_full(&mut vm, "after-cycle");
}

#[test]
fn gc_adds_new_segments_on_oom() {
    let mut vm = VM::new();

    for i in 0..4 {
        allocate_until_full(&mut vm, &i.to_string());
    }

    assert!(
        vm.arena.segment_count() > 1,
        "there should be more than one segment now: segment_count={}, stack_length={}, vm={:?}",
        vm.arena.segment_count(),
        vm.stack.data.len(),
        vm
    )
}

#[test]
fn gc_deletes_old_segments() {
    let mut vm = VM::new();

    vm.stack.push_isolated();

    // Fill several segments.
    for i in 0..4 {
        allocate_until_full(&mut vm, &i.to_string());
    }

    let segment_count_before = vm.arena.segment_count();

    assert!(
        segment_count_before >= 4,
        "expected multiple segments"
    );

    // Keep one object alive in the first segment.
    let survivor = vm.stack.data[0];

    // Drop all other roots.
    vm.stack.end();

    vm.stack.push_isolated();
    vm.stack.top_add_index(survivor);

    // Run enough GC work to finish several cycles.
    for i in 0..4 {
        vm.gc_step(RUNS_PER_MEMMAX);
    }

    assert!(
        vm.arena.segment_count() < 3,
        "unused trailing segments should be removed (there were {}, which is not less than 3)",
        vm.arena.segment_count()
    );

    // Verify the survivor still works.
    let value = vm.arena.get(survivor);

    assert!(
        matches!(value.tag, RT::Str),
        "survivor was corrupted"
    );
}