//! The close-code allow list, including the codes that may never be sent.

use tetherscript::websocket::close;
use tetherscript::websocket::close_code;
use tetherscript::websocket::error::ProtocolError;

#[test]
fn the_locally_generated_close_codes_are_never_accepted() {
    // 1005 = no status, 1006 = abnormal closure, 1015 = TLS handshake failure.
    // All three describe events that cannot be reported by a frame that arrived.
    for code in [1005u16, 1006, 1015] {
        assert_eq!(
            close_code::check(code),
            Err(ProtocolError::ForbiddenCloseCode { code }),
            "close code {code} must be rejected on the wire"
        );
        assert_eq!(
            close::validate(&code.to_be_bytes()),
            Err(ProtocolError::ForbiddenCloseCode { code })
        );
    }
}

#[test]
fn the_registered_and_private_close_codes_are_accepted() {
    for code in [1000u16, 1001, 1002, 1003, 1007, 1010, 1011, 1014, 3000, 4999] {
        assert!(close_code::check(code).is_ok(), "code {code} should be ok");
    }
}

#[test]
fn out_of_range_and_reserved_close_codes_are_rejected() {
    // 0..=999 unassigned, 1004 reserved, 1016..=2999 reserved for future RFCs,
    // 5000.. outside every registry.
    for code in [0u16, 999, 1004, 1016, 2999, 5000, u16::MAX] {
        assert_eq!(
            close_code::check(code),
            Err(ProtocolError::ForbiddenCloseCode { code })
        );
    }
}
