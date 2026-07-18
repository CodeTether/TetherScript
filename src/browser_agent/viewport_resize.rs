use crate::browser_agent::page::BrowserPage;

impl BrowserPage {
    /// Set viewport dimensions in CSS pixels and notify the page runtime.
    ///
    /// # Arguments
    ///
    /// * `width` - New viewport width in CSS pixels.
    /// * `height` - New viewport height in CSS pixels.
    ///
    /// # Returns
    ///
    /// `Ok(())` after synchronizing Rust and JavaScript viewport state.
    ///
    /// # Errors
    ///
    /// Returns `Err` for non-positive dimensions or JavaScript runtime failures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::browser_agent::BrowserPage;
    ///
    /// let mut page = BrowserPage::from_html("mem://docs", "<main>Docs</main>");
    /// page.set_viewport_size(320, 640).unwrap();
    /// assert_eq!(page.viewport().width, 320);
    /// ```
    pub fn set_viewport_size(&mut self, width: i64, height: i64) -> Result<(), String> {
        if width <= 0 || height <= 0 {
            return Err("viewport width and height must be positive".into());
        }
        self.eval_js(&format!("resizeTo({width},{height})"))?;
        self.viewport_width = width;
        self.viewport_height = height;
        Ok(())
    }
}
