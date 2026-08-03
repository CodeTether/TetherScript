//! Recursion control for nested macro expansion.
//!
//! A macro must be able to call another macro — the reference views nest layout components
//! several deep — so expansion is inherently recursive and needs a bound.
//!
//! # The bound
//!
//! **16 nested macro expansions.** That matches the engine's own include-depth limit
//! (`template_include` / `template_macro_call::MAX_DEPTH`), so a template that is legal for
//! includes is legal for macros, and the two cannot be combined to exceed either. Sixteen
//! is far above the deepest real nesting in the reference application and far below any
//! stack risk.
//!
//! # Direct and indirect self-recursion
//!
//! Both are caught, and by a sharper rule than the depth bound alone: a call whose
//! `namespace::name` already appears on the active frame stack is rejected immediately,
//! with the whole chain named. `a → a` is direct; `a → b → a` is indirect; the stack check
//! sees both identically.
//!
//! This is sound because the engine has no arithmetic and no `{% set %}`, so a macro cannot
//! shrink an argument toward a base case: a body that reaches itself has no way to stop, and
//! the recursion is infinite by construction rather than merely deep. The depth bound
//! remains as the backstop for a chain of 17 *distinct* macros.

/// Maximum nested macro expansions, matching the engine's include depth limit.
pub const MAX_DEPTH: usize = 16;

/// The stack of macro call paths currently being expanded, outermost first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frames(Vec<String>);

impl Frames {
    /// An empty stack, for the outermost call.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::tmplmacro::frames::Frames;
    ///
    /// assert_eq!(Frames::new().depth(), 0);
    /// ```
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Number of expansions currently active.
    pub fn depth(&self) -> usize {
        self.0.len()
    }

    /// Push a resolved call key, rejecting a cycle or an over-deep chain.
    ///
    /// # Arguments
    ///
    /// * `key` — Fully resolved `namespace::name` of the macro about to be expanded.
    ///
    /// # Returns
    ///
    /// A new stack with `key` appended; the receiver is left untouched so sibling calls at
    /// the same level do not accumulate each other's frames.
    ///
    /// # Errors
    ///
    /// Returns an error naming the full chain when `key` is already active (direct or
    /// indirect self-recursion), or when the chain would exceed [`MAX_DEPTH`].
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::tmplmacro::frames::Frames;
    ///
    /// let one = Frames::new().push("ui::a").unwrap();
    /// let two = one.push("ui::b").unwrap();
    /// assert_eq!(two.depth(), 2);
    /// // indirect: a -> b -> a
    /// assert!(two.push("ui::a").is_err());
    /// // direct: a -> a
    /// assert!(one.push("ui::a").is_err());
    /// ```
    pub fn push(&self, key: &str) -> Result<Self, String> {
        if self.0.iter().any(|active| active.as_str() == key) {
            return Err(format!(
                "template: macro `{key}` calls itself; expansion chain was {} -> {key}",
                self.0.join(" -> ")
            ));
        }
        if self.0.len() >= MAX_DEPTH {
            return Err(format!(
                "template: macro expansion exceeded the limit of {MAX_DEPTH} nested calls at \
                 `{key}`; chain was {}",
                self.0.join(" -> ")
            ));
        }
        let mut next = self.0.clone();
        next.push(key.to_string());
        Ok(Self(next))
    }
}
