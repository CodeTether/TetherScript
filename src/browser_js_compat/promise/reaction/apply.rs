use super::super::*;

pub(super) fn settle(
    kind: super::types::Kind,
    current: state::PromiseState,
) -> state::PromiseState {
    match kind {
        super::types::Kind::Then { ok, err } => then::settle(ok, err, current),
        super::types::Kind::Finally { callback } => finally::settle(callback, current),
    }
}
