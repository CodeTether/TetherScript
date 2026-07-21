const browser = {
  browser_compatibility_report: ['browser_compatibility_report()', 'Report supported browser runtime features.'],
  browser_display_list: ['browser_display_list(html[, css[, width]])', 'Build a browser paint display list.'],
  browser_eval_js: ['browser_eval_js(html, script)', 'Evaluate JavaScript against an HTML document.'],
  browser_layout: ['browser_layout(html[, css[, width]])', 'Compute browser layout boxes.'],
  browser_parse_css: ['browser_parse_css(css)', 'Parse CSS into runtime values.'],
  browser_parse_html: ['browser_parse_html(html)', 'Parse HTML into runtime values.'],
  browser_query_selector: ['browser_query_selector(html, selector)', 'Query the first matching HTML element.'],
  browser_raster: ['browser_raster(html[, css[, width]])', 'Rasterize a browser document.'],
  browser_render: ['browser_render(html[, css[, width]])', 'Render a browser document.'],
  browser_render_ppm: ['browser_render_ppm(html[, css[, width]])', 'Render a browser document as PPM bytes.'],
  browser_run_scripts: ['browser_run_scripts(html)', 'Run scripts embedded in an HTML document.'],
  browser_run_scripts_at: ['browser_run_scripts_at(html, url)', 'Run embedded scripts with a document URL.'],
  browser_snapshot: ['browser_snapshot(html[, css[, width]])', 'Create a browser document snapshot.'],
  browser_styles: ['browser_styles(html[, css])', 'Compute styles for an HTML document.'],
  browser_text_content: ['browser_text_content(html, selector)', 'Return text from the first matching element.'],
  js_eval: ['js_eval(source)', 'Evaluate JavaScript and return a tetherscript value.'],
};

module.exports = { browser };
