use super::{super::*, types::Reaction};

pub(super) fn settle(item: Reaction, current: state::PromiseState) {
    let next = match item.kind {
        super::types::Kind::Then { ok, err } => then::settle(ok, err, current),
        super::types::Kind::Finally { callback } => {
            return finally::settle_reaction(callback, current, item.state, item.object, item.queue)
        }
    };
    super::settle(&item.state, &item.object, &item.queue, next);
}
