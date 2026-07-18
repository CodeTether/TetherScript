//! Deterministic WebGL rendering state.

pub(super) const MAX_COMMANDS: usize = 64;

#[derive(Clone)]
pub(super) struct WebGlState {
    pub version: u8,
    pub width: u32,
    pub height: u32,
    pub viewport: [i64; 4],
    pub clear_color: [f64; 4],
    pub scissor_box: [i64; 4],
    pub scissor_test: bool,
    pub color_mask: [bool; 4],
    pub errors: Vec<u32>,
    pub commands: Vec<String>,
    pub pipeline: super::webgl_pipeline::State,
}

impl WebGlState {
    pub fn new(version: u8, width: u32, height: u32) -> Self {
        Self {
            version,
            width,
            height,
            viewport: [0, 0, width as i64, height as i64],
            clear_color: [0.0, 0.0, 0.0, 0.0],
            scissor_box: [0, 0, width as i64, height as i64],
            scissor_test: false,
            color_mask: [true; 4],
            errors: Vec::new(),
            commands: Vec::new(),
            pipeline: super::webgl_pipeline::State::default(),
        }
    }

    pub fn push(&mut self, command: String) {
        if self.commands.len() < MAX_COMMANDS {
            self.commands.push(command);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}
