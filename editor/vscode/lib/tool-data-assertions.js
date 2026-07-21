const assertions = {
  assert_enabled: ['assert_enabled(browser, selector)', 'Assert that a browser element is enabled.'],
  assert_no_console_errors: ['assert_no_console_errors(browser)', 'Assert that the browser has no console errors.'],
  assert_no_failed_requests: ['assert_no_failed_requests(browser)', 'Assert that the browser has no failed requests.'],
  assert_react_component: ['assert_react_component(browser, name)', 'Assert that a React component exists.'],
  assert_route: ['assert_route(browser, url)', 'Assert that the browser reached a URL.'],
  assert_screenshot_matches: ['assert_screenshot_matches(browser, expected)', 'Assert that a browser screenshot matches.'],
  assert_selector: ['assert_selector(browser, selector)', 'Assert that a browser selector matches.'],
  assert_text: ['assert_text(browser, text)', 'Assert that browser text appears.'],
  assert_visible: ['assert_visible(browser, selector)', 'Assert that a browser element is visible.'],
};

module.exports = { assertions };
