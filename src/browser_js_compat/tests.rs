use super::super::*;

macro_rules! compat_test {
    ($name:ident, $path:literal) => {
        #[path = $path]
        mod $name;
    };
}

compat_test!(tests_abort_signal, "tests_abort_signal.rs");
compat_test!(tests_blob_file, "tests_blob_file.rs");
compat_test!(tests_blob_file_bytes, "tests_blob_file_bytes.rs");
compat_test!(tests_body_array_buffer, "tests_body_array_buffer.rs");
compat_test!(tests_clipboard_item, "tests_clipboard_item.rs");
compat_test!(tests_crypto, "tests_crypto.rs");
compat_test!(tests_dom_exception, "tests_dom_exception.rs");
compat_test!(tests_file_reader_abort, "tests_file_reader_abort.rs");
compat_test!(tests_file_reader, "tests_file_reader.rs");
compat_test!(tests_fr_ab, "tests_file_reader_array_buffer.rs");
compat_test!(tests_form_data, "tests_form_data.rs");
compat_test!(tests_headers, "tests_headers.rs");
compat_test!(tests_history_state, "tests_history_state.rs");
compat_test!(tests_history_traversal, "tests_history_traversal.rs");
compat_test!(tests_navigator, "tests_navigator.rs");
compat_test!(tests_request, "tests_request.rs");
compat_test!(tests_response, "tests_response.rs");
compat_test!(tests_response_static, "tests_response_static.rs");
compat_test!(tests_structured, "tests_structured.rs");
compat_test!(tests_text, "tests_text.rs");
compat_test!(tests_text_decoder, "tests_text_decoder.rs");
compat_test!(tests_url_search_params, "tests_url_search_params.rs");
compat_test!(tests_url_static, "tests_url_static.rs");

#[test]
fn crypto_random_values_are_seedable_and_mutate_uint8_array() {
    let result = eval_with_dom(
        "<main></main>",
        "crypto.setRandomSeed(7); let a=Uint8Array(4); crypto.getRandomValues(a); let first=a.join('-'); crypto.setRandomSeed(7); let b=Uint8Array(4); crypto.getRandomValues(b); first == b.join('-') && first != '0-0-0-0' && a.length == 4;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::Bool(true));
}

#[test]
fn crypto_subtle_digest_sha256_returns_byte_array_promise() {
    let result = eval_with_dom(
        "<main></main>",
        "let out=''; crypto.subtle.digest('SHA-256', TextEncoder().encode('abc')).then(function(hash){ out=hash.length + ':' + hash[0] + ':' + hash[1] + ':' + hash[2] + ':' + hash[3]; }); out;",
    )
    .unwrap();
    assert_eq!(result.value, JsValue::String("32:186:120:22:191".into()));
}
