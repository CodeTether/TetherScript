//! WebGL sticky-error mapping for rejected buffer sources.

use super::*;

pub(super) fn record(state: &mut WebGlState, error: source::Error) {
    let code = match error {
        source::Error::Invalid => webgl_constants::INVALID_VALUE,
        source::Error::OutOfMemory => webgl_constants::OUT_OF_MEMORY,
    };
    webgl_error::record(state, code);
}
