//! The [`Span`] primitive: a half-open byte range `[start, end)` into a source
//! buffer.
//!
//! Spans hold **byte offsets only**. They carry no line, no column, and no
//! reference to the text, which is what makes them cheap to store on every
//! token and cheap to combine while parsing. Rendering them is
//! [`crate::diagnostic::SourceMap`]'s job.
//!
//! A span with `start == end` is *zero-width*: it points *between* two
//! characters. That is the honest representation for "expected `)` here" and
//! for end-of-file errors, where no source character is at fault.

/// A half-open byte range `[start, end)` inside a source buffer.
///
/// # Examples
///
/// ```rust
/// use tetherscript::diagnostic::Span;
///
/// let s = Span::new(4, 7);
/// assert_eq!((s.start, s.end), (4, 7));
/// assert_eq!(s.len(), 3);
/// assert!(!s.is_empty());
/// assert!(s.contains(4) && s.contains(6) && !s.contains(7));
///
/// // Zero-width spans point between characters.
/// let eof = Span::at(12);
/// assert!(eof.is_empty());
/// assert_eq!(eof.len(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset. Never less than `start`.
    pub end: usize,
}

impl Span {
    /// Creates a span covering `[start, end)`.
    ///
    /// # Arguments
    ///
    /// * `start` — inclusive start byte offset.
    /// * `end` — exclusive end byte offset. If `end < start` the arguments are
    ///   swapped, so a `Span` is always well ordered and callers never have to
    ///   defend against inverted ranges.
    ///
    /// # Returns
    ///
    /// A normalised `Span`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Span;
    /// assert_eq!(Span::new(9, 3), Span::new(3, 9));
    /// ```
    pub fn new(start: usize, end: usize) -> Self {
        if end < start {
            Self {
                start: end,
                end: start,
            }
        } else {
            Self { start, end }
        }
    }

    /// Creates a zero-width span at `offset`.
    ///
    /// # Arguments
    ///
    /// * `offset` — the byte offset to point between characters at.
    ///
    /// # Returns
    ///
    /// A `Span` where `start == end == offset`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Span;
    /// let s = Span::at(5);
    /// assert!(s.is_empty());
    /// assert_eq!(s.start, 5);
    /// ```
    pub fn at(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Length of the span in **bytes** (not characters, not columns).
    ///
    /// # Returns
    ///
    /// `end - start`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Span;
    /// // "é" is two bytes, so a one-character span has byte length 2.
    /// assert_eq!(Span::new(0, 2).len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether the span is zero-width.
    ///
    /// # Returns
    ///
    /// `true` when `start == end`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Span;
    /// assert!(Span::at(3).is_empty());
    /// assert!(!Span::new(3, 4).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Whether `offset` falls inside the span.
    ///
    /// # Arguments
    ///
    /// * `offset` — byte offset to test.
    ///
    /// # Returns
    ///
    /// `true` if `start <= offset < end`. A zero-width span contains nothing,
    /// which keeps `contains` consistent with the half-open convention.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Span;
    /// assert!(Span::new(2, 4).contains(3));
    /// assert!(!Span::at(3).contains(3));
    /// ```
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Smallest span covering both `self` and `other`.
    ///
    /// This is how the parser grows a span while it consumes tokens: join the
    /// first token's span with the last one's.
    ///
    /// # Arguments
    ///
    /// * `other` — the span to absorb.
    ///
    /// # Returns
    ///
    /// A span from `min(starts)` to `max(ends)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tetherscript::diagnostic::Span;
    /// assert_eq!(Span::new(2, 4).join(Span::new(9, 11)), Span::new(2, 11));
    /// ```
    pub fn join(&self, other: Span) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
