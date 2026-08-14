//! Unicode text collection from the native window callback.

use std::{cell::RefCell, rc::Rc};

use minifb::InputCallback;

pub(super) struct Text {
    characters: Rc<RefCell<Vec<char>>>,
}

impl Text {
    pub(super) fn new(characters: Rc<RefCell<Vec<char>>>) -> Self {
        Self { characters }
    }
}

impl InputCallback for Text {
    fn add_char(&mut self, codepoint: u32) {
        if let Some(character) = char::from_u32(codepoint) {
            if !character.is_control() {
                self.characters.borrow_mut().push(character);
            }
        }
    }
}
