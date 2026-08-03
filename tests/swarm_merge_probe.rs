//! Probe test that verifies swarm worktree merge.
//!
//! This file is created by a swarm worker inside a managed worktree. Its only
//! purpose is to prove that the worktree's changes are merged back into the
//! main workspace: if `probe` is present and passes here, the merge worked.

#[test]
fn probe() {
    assert_eq!(1 + 1, 2);
}
