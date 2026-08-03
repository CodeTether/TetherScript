//! Unit tests for the runtime `&mut` exclusivity layer (`src/borrow_runtime.rs`).
//!
//! These exercise the borrow state on its own — no interpreter, no VM — because
//! the module is deliberately self-contained so its rules are testable without a
//! backend threaded through them.

use std::cell::RefCell;
use std::rc::Rc;

use tetherscript::borrow_runtime::{
    heap_id_of, BorrowError, BorrowKind, BorrowState, BorrowTable, BorrowTracker, HeapId,
};
use tetherscript::value::Value;

fn list(len: usize) -> Value {
    Value::List(Rc::new(RefCell::new(
        (0..len as i64).map(Value::Int).collect(),
    )))
}

// ---------- shared + shared is allowed ----------

#[test]
fn shared_borrows_compose() {
    let mut state = BorrowState::default();

    state.acquire(BorrowKind::Shared, "items").unwrap();
    state.acquire(BorrowKind::Shared, "items").unwrap();
    state.acquire(BorrowKind::Shared, "items").unwrap();

    assert_eq!(state.shared_count(), 3);
    assert!(!state.is_mutably_borrowed());
    assert!(state.invariant_holds());
}

// ---------- shared + mutable is rejected, both orders ----------

#[test]
fn mutable_borrow_rejected_while_shared_borrow_live() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Shared, "items").unwrap();

    let err = state.acquire(BorrowKind::Mutable, "items").unwrap_err();

    assert_eq!(
        err,
        BorrowError::Conflict {
            binding: "items".to_string(),
            requested: BorrowKind::Mutable,
        }
    );
    assert_eq!(
        err.to_string(),
        "cannot mutably borrow `items` while it is already borrowed"
    );
    assert_eq!(state.shared_count(), 1, "failed acquire must not mutate");
    assert!(state.invariant_holds());
}

#[test]
fn shared_borrow_rejected_while_mutable_borrow_live() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Mutable, "items").unwrap();

    let err = state.acquire(BorrowKind::Shared, "items").unwrap_err();

    assert_eq!(
        err.to_string(),
        "cannot borrow `items` while it is mutably borrowed"
    );
    assert_eq!(state.shared_count(), 0);
    assert!(state.is_mutably_borrowed());
    assert!(state.invariant_holds());
}

// ---------- mutable + mutable is rejected ----------

#[test]
fn second_mutable_borrow_rejected() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Mutable, "buf").unwrap();

    let err = state.acquire(BorrowKind::Mutable, "buf").unwrap_err();

    assert_eq!(
        err.to_string(),
        "cannot mutably borrow `buf` while it is already borrowed"
    );
    assert!(state.invariant_holds());
}

// ---------- release restores availability ----------

#[test]
fn releasing_last_shared_borrow_permits_mutable_borrow() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Shared, "items").unwrap();
    state.acquire(BorrowKind::Shared, "items").unwrap();

    state.release(BorrowKind::Shared, "items").unwrap();
    assert!(
        state.acquire(BorrowKind::Mutable, "items").is_err(),
        "one shared borrow is still live"
    );

    state.release(BorrowKind::Shared, "items").unwrap();
    state.acquire(BorrowKind::Mutable, "items").unwrap();

    assert!(state.is_mutably_borrowed());
    assert!(state.invariant_holds());
}

#[test]
fn releasing_mutable_borrow_permits_shared_borrow() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Mutable, "items").unwrap();
    state.release(BorrowKind::Mutable, "items").unwrap();

    state.acquire(BorrowKind::Shared, "items").unwrap();

    assert!(!state.is_mutably_borrowed());
    assert_eq!(state.shared_count(), 1);
    assert!(state.invariant_holds());
}

// ---------- unbalanced release is detected, not absorbed ----------

#[test]
fn releasing_unheld_shared_borrow_is_reported() {
    let mut state = BorrowState::default();

    let err = state.release(BorrowKind::Shared, "items").unwrap_err();

    assert_eq!(
        err,
        BorrowError::UnbalancedRelease {
            binding: "items".to_string(),
            requested: BorrowKind::Shared,
        }
    );
    assert!(err.to_string().contains("`items`"));
    assert_eq!(state.shared_count(), 0, "count must not underflow or wrap");
    assert!(state.invariant_holds());
}

#[test]
fn releasing_wrong_kind_is_reported_and_leaves_state_intact() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Shared, "items").unwrap();

    let err = state.release(BorrowKind::Mutable, "items").unwrap_err();

    assert!(matches!(err, BorrowError::UnbalancedRelease { .. }));
    assert_eq!(state.shared_count(), 1, "shared borrow must still be live");
    assert!(!state.is_mutably_borrowed());
    assert!(state.invariant_holds());
}

// ---------- move / borrow interaction ----------

#[test]
fn cannot_move_while_shared_borrow_live() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Shared, "xs").unwrap();

    let err = state.mark_moved("xs").unwrap_err();

    assert_eq!(err.to_string(), "cannot move `xs` while it is borrowed");
    assert!(!state.is_moved());
    assert!(state.invariant_holds());
}

#[test]
fn cannot_move_while_mutable_borrow_live() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Mutable, "xs").unwrap();

    assert_eq!(
        state.mark_moved("xs").unwrap_err().to_string(),
        "cannot move `xs` while it is borrowed"
    );
    assert!(!state.is_moved());
}

#[test]
fn moved_value_is_not_borrowable() {
    let mut state = BorrowState::default();
    state.mark_moved("xs").unwrap();

    for kind in [BorrowKind::Shared, BorrowKind::Mutable] {
        let err = state.acquire(kind, "xs").unwrap_err();
        assert_eq!(err.to_string(), "cannot borrow moved value `xs`");
    }
    assert!(state.invariant_holds());
}

#[test]
fn move_after_all_borrows_released_is_allowed() {
    let mut state = BorrowState::default();
    state.acquire(BorrowKind::Shared, "xs").unwrap();
    state.release(BorrowKind::Shared, "xs").unwrap();

    state.mark_moved("xs").unwrap();

    assert!(state.is_moved());
    assert!(state.invariant_holds());
}

#[test]
fn reset_clears_moved_and_borrow_accounting() {
    let mut state = BorrowState::default();
    state.mark_moved("xs").unwrap();

    state.reset();

    state.acquire(BorrowKind::Mutable, "xs").unwrap();
    assert!(!state.is_moved());
    assert!(state.invariant_holds());
}

// ---------- guard Drop ----------

#[test]
fn guard_drop_releases_borrow_on_early_return() {
    let tracker = BorrowTracker::new();
    let id = HeapId(0xBEEF);

    fn bail(tracker: &BorrowTracker, id: HeapId) -> Result<(), String> {
        let _guard = tracker
            .borrow_value(id, BorrowKind::Mutable, "items")
            .map_err(|err| err.to_string())?;
        Err("early return before any manual release".to_string())
    }

    assert!(bail(&tracker, id).is_err());

    // If the count had leaked, this correct code would fail.
    tracker
        .borrow_value(id, BorrowKind::Mutable, "items")
        .expect("guard Drop must have released the borrow");
}

#[test]
fn guard_drop_releases_borrow_on_question_mark_propagation() {
    let tracker = BorrowTracker::new();
    let id = HeapId(0xC0DE);

    fn inner(tracker: &BorrowTracker, id: HeapId) -> Result<(), BorrowError> {
        let _outer = tracker.borrow_value(id, BorrowKind::Shared, "items")?;
        // This `?` propagates and unwinds `_outer` on the way out.
        let _conflict = tracker.borrow_value(id, BorrowKind::Mutable, "items")?;
        Ok(())
    }

    assert!(inner(&tracker, id).is_err());

    tracker.with_table(|table| {
        let state = table.state(id).expect("entry exists");
        assert_eq!(state.shared_count(), 0, "propagated `?` must not leak");
        assert!(!state.is_mutably_borrowed());
        assert!(state.invariant_holds());
    });
}

#[test]
fn guard_reports_its_own_provenance() {
    let tracker = BorrowTracker::new();
    let guard = tracker
        .borrow_value(HeapId(7), BorrowKind::Shared, "items")
        .unwrap();

    assert_eq!(guard.binding(), "items");
    assert_eq!(guard.kind(), BorrowKind::Shared);
    assert_eq!(guard.heap_id(), HeapId(7));
}

#[test]
fn double_release_via_table_after_guard_drop_is_an_accounting_fault() {
    let tracker = BorrowTracker::new();
    let id = HeapId(0x5150);

    let guard = tracker
        .borrow_value(id, BorrowKind::Shared, "items")
        .unwrap();
    // Simulate a backend that releases by hand *and* lets the guard drop.
    tracker
        .with_table(|table| table.release(id, BorrowKind::Shared, "items"))
        .unwrap();
    drop(guard);

    assert_eq!(
        tracker.with_table(|table| table.accounting_faults()),
        1,
        "the redundant release must be tallied, not silently absorbed"
    );
    // And the state is still sane: nothing wedged as permanently borrowed.
    tracker
        .borrow_value(id, BorrowKind::Mutable, "items")
        .expect("value must not be wedged");
}

// ---------- heap identity: the aliases the static pass cannot see ----------

#[test]
fn scalars_have_no_heap_id_and_need_no_tracking() {
    for value in [
        Value::Nil,
        Value::Int(1),
        Value::Float(1.5),
        Value::Bool(true),
    ] {
        assert!(
            heap_id_of(&value).is_none(),
            "{} is Copy; tracking it would be pointless",
            value.type_name()
        );
    }
}

#[test]
fn borrowing_a_scalar_yields_no_guard_and_never_conflicts() {
    let tracker = BorrowTracker::new();

    let first = tracker
        .borrow_named(&Value::Int(9), BorrowKind::Mutable, "n")
        .unwrap();
    let second = tracker
        .borrow_named(&Value::Int(9), BorrowKind::Mutable, "n")
        .unwrap();

    assert!(first.is_none());
    assert!(second.is_none());
    assert_eq!(tracker.with_table(|table| table.tracked()), 0);
}

#[test]
fn value_clone_is_an_alias_and_shares_borrow_state() {
    let items = list(3);
    let alias = items.clone();

    assert_eq!(heap_id_of(&items), heap_id_of(&alias));

    let tracker = BorrowTracker::new();
    let _held = tracker
        .borrow_named(&items, BorrowKind::Shared, "items")
        .unwrap();

    let err = tracker
        .borrow_named(&alias, BorrowKind::Mutable, "alias")
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "cannot mutably borrow `alias` while it is already borrowed"
    );
}

#[test]
fn distinct_allocations_do_not_interfere() {
    let a = list(1);
    let b = list(1);
    assert_eq!(a, b, "equal by value");
    assert_ne!(heap_id_of(&a), heap_id_of(&b), "but distinct allocations");

    let tracker = BorrowTracker::new();
    let _mutable = tracker
        .borrow_named(&a, BorrowKind::Mutable, "a")
        .unwrap()
        .unwrap();

    tracker
        .borrow_named(&b, BorrowKind::Shared, "b")
        .expect("an unrelated value must stay borrowable");
}

// ---------- reentrancy cases from borrow_runtime_reentrancy.rs ----------

#[test]
fn case_1_mutating_a_list_while_iterating_it_is_caught() {
    let tracker = BorrowTracker::new();
    let xs = list(3);

    // `for x in &xs` holds a shared borrow for the whole loop.
    let _iteration = tracker
        .borrow_named(&xs, BorrowKind::Shared, "xs")
        .unwrap()
        .unwrap();

    // `xs.push(..)` inside the body wants exclusivity.
    let err = tracker
        .borrow_named(&xs, BorrowKind::Mutable, "xs")
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "cannot mutably borrow `xs` while it is already borrowed"
    );
}

#[test]
fn case_2_closure_capture_conflicting_with_enclosing_borrow_is_caught() {
    let tracker = BorrowTracker::new();
    let cell = list(1);
    let captured = cell.clone(); // what a closure capture actually is

    let _view = tracker
        .borrow_named(&cell, BorrowKind::Shared, "cell")
        .unwrap()
        .unwrap();

    let err = tracker
        .borrow_named(&captured, BorrowKind::Mutable, "cell")
        .unwrap_err();

    assert_eq!(
        err.to_string(),
        "cannot mutably borrow `cell` while it is already borrowed"
    );
}

#[test]
fn case_3_callee_mut_borrow_while_caller_holds_shared_is_caught() {
    let tracker = BorrowTracker::new();
    let items = list(2);

    fn mutate(tracker: &BorrowTracker, arg: &Value, name: &str) -> Result<(), BorrowError> {
        let _exclusive = tracker.borrow_named(arg, BorrowKind::Mutable, name)?;
        Ok(())
    }

    let _caller_view = tracker
        .borrow_named(&items, BorrowKind::Shared, "items")
        .unwrap()
        .unwrap();

    let err = mutate(&tracker, &items, "items").unwrap_err();

    assert_eq!(
        err.to_string(),
        "cannot mutably borrow `items` while it is already borrowed"
    );
}

#[test]
fn case_5_recursive_shared_borrows_are_allowed() {
    let tracker = BorrowTracker::new();
    let node = list(1);

    fn walk(tracker: &BorrowTracker, node: &Value, depth: usize) -> Result<(), BorrowError> {
        let _frame = tracker.borrow_named(node, BorrowKind::Shared, "node")?;
        if depth > 0 {
            walk(tracker, node, depth - 1)?;
        }
        Ok(())
    }

    walk(&tracker, &node, 8).expect("shared borrows compose across frames");

    let id = heap_id_of(&node).unwrap();
    tracker.with_table(|table| {
        let state = table.state(id).expect("entry exists");
        assert_eq!(state.shared_count(), 0, "every frame released on unwind");
    });
}

// ---------- move via the value-level API ----------

#[test]
fn move_named_refuses_while_borrowed_and_then_blocks_reborrow() {
    let tracker = BorrowTracker::new();
    let xs = list(1);

    let guard = tracker
        .borrow_named(&xs, BorrowKind::Shared, "xs")
        .unwrap()
        .unwrap();
    assert_eq!(
        tracker.move_named(&xs, "xs").unwrap_err().to_string(),
        "cannot move `xs` while it is borrowed"
    );

    drop(guard);
    tracker.move_named(&xs, "xs").unwrap();

    assert_eq!(
        tracker
            .borrow_named(&xs, BorrowKind::Shared, "xs")
            .unwrap_err()
            .to_string(),
        "cannot borrow moved value `xs`"
    );
}

#[test]
fn moving_a_scalar_is_a_no_op() {
    let tracker = BorrowTracker::new();

    tracker.move_named(&Value::Int(3), "n").unwrap();

    // Still borrowable: scalars are Copy, so a move clones.
    assert!(tracker
        .borrow_named(&Value::Int(3), BorrowKind::Mutable, "n")
        .unwrap()
        .is_none());
}

// ---------- table housekeeping ----------

#[test]
fn forget_drops_the_entry_so_the_table_does_not_grow_forever() {
    let tracker = BorrowTracker::new();
    let id = HeapId(0x1234);

    drop(tracker.borrow_value(id, BorrowKind::Shared, "tmp").unwrap());
    assert_eq!(tracker.with_table(|table| table.tracked()), 1);

    tracker.forget(id);

    assert_eq!(tracker.with_table(|table| table.tracked()), 0);
    assert!(!tracker.is_moved(id));
}

#[test]
fn table_state_is_not_created_by_a_read() {
    let table = BorrowTable::default();

    assert!(table.state(HeapId(1)).is_none());
    assert_eq!(table.tracked(), 0);
}

#[test]
fn error_binding_accessor_names_the_offending_identifier() {
    let errors = [
        BorrowError::Conflict {
            binding: "a".to_string(),
            requested: BorrowKind::Mutable,
        },
        BorrowError::Moved {
            binding: "a".to_string(),
        },
        BorrowError::MoveWhileBorrowed {
            binding: "a".to_string(),
        },
        BorrowError::UnbalancedRelease {
            binding: "a".to_string(),
            requested: BorrowKind::Shared,
        },
    ];

    for err in errors {
        assert_eq!(err.binding(), "a");
        assert!(err.to_string().contains("`a`"), "{err}");
    }
}
