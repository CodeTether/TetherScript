//! Typed WebGL metadata records.

/// One deterministic WebGL metadata command captured by the host.
///
/// # Examples
///
/// ```
/// use tetherscript::browser_agent::WebGlCommand;
///
/// let command = WebGlCommand {
///     operation: "clear".into(),
///     args: vec!["16384".into()],
/// };
/// assert_eq!(command.operation, "clear");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebGlCommand {
    /// WebGL operation name such as `viewport`, `clearColor`, or `clear`.
    pub operation: String,
    /// Stringified command arguments in call order.
    pub args: Vec<String>,
}

/// Deterministic state snapshot of one software-rendered WebGL canvas context.
///
/// # Examples
///
/// ```
/// use tetherscript::browser_agent::WebGlContextSnapshot;
///
/// let snapshot = WebGlContextSnapshot {
///     version: 1,
///     width: 300,
///     height: 150,
///     viewport: [0, 0, 300, 150],
///     clear_color: [0.0, 0.0, 0.0, 0.0],
///     scissor_box: [0, 0, 300, 150],
///     scissor_test: false,
///     color_mask: [true; 4],
///     supported_extensions: Vec::new(),
///     commands: Vec::new(),
/// };
/// assert_eq!(snapshot.version, 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct WebGlContextSnapshot {
    /// WebGL major version, `1` for `webgl`, `2` for `webgl2`.
    pub version: u8,
    /// Backing canvas width.
    pub width: u32,
    /// Backing canvas height.
    pub height: u32,
    /// Current viewport `[x, y, width, height]`.
    pub viewport: [i64; 4],
    /// Current clear color `[r, g, b, a]`.
    pub clear_color: [f64; 4],
    /// Current scissor rectangle `[x, y, width, height]` in drawing-buffer coordinates.
    pub scissor_box: [i64; 4],
    /// Whether the scissor test currently restricts raster operations.
    pub scissor_test: bool,
    /// Per-channel color write mask in red, green, blue, alpha order.
    pub color_mask: [bool; 4],
    /// Deterministic supported extension names.
    pub supported_extensions: Vec<String>,
    /// Bounded WebGL command log.
    pub commands: Vec<WebGlCommand>,
}
