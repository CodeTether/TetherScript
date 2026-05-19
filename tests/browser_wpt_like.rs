#[path = "browser_wpt_like/mod.rs"]
mod fixtures;

macro_rules! fixture_test {
    ($name:ident, $module:ident) => {
        #[test]
        fn $name() {
            fixtures::$module::run();
        }
    };
}

fixture_test!(dom_events_fixture_subset, dom_events);
fixture_test!(selectors_fixture_subset, selectors);
fixture_test!(fetch_cors_fixture_subset, fetch_cors);
fixture_test!(modules_fixture_subset, modules);
fixture_test!(css_layout_fixture_subset, css_layout);
fixture_test!(timers_microtasks_fixture_subset, timers_microtasks);
fixture_test!(storage_fixture_subset, storage);
fixture_test!(html_tree_fixture_subset, html_tree);
fixture_test!(forms_fixture_subset, forms);
fixture_test!(navigation_history_fixture_subset, navigation_history);
fixture_test!(storage_context_fixture_subset, storage_context);
fixture_test!(keyboard_pointer_fixture_subset, keyboard_pointer);
fixture_test!(focus_fixture_subset, focus);
fixture_test!(file_transfer_fixture_subset, file_transfer);
fixture_test!(realtime_fixture_subset, realtime);
fixture_test!(permissions_media_fixture_subset, permissions_media);
fixture_test!(dialog_clipboard_fixture_subset, dialog_clipboard);
fixture_test!(frames_fixture_subset, frames);
fixture_test!(security_policy_fixture_subset, security_policy);
fixture_test!(canvas_webgl_fixture_subset, canvas_webgl);
fixture_test!(accessibility_fixture_subset, accessibility);
fixture_test!(service_worker_cache_fixture_subset, service_worker_cache);
fixture_test!(indexed_db_fixture_subset, indexed_db);
fixture_test!(selection_fixture_subset, selection);
fixture_test!(visual_diff_fixture_subset, visual_diff);
fixture_test!(trace_persistence_fixture_subset, trace_persistence);
fixture_test!(selectors_error_fixture_subset, selectors_errors);
fixture_test!(fetch_cors_error_fixture_subset, fetch_cors_errors);
fixture_test!(modules_error_fixture_subset, modules_errors);
fixture_test!(html_tree_unsupported_fixture_subset, html_tree_unsupported);
