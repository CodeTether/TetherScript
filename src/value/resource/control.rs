//! Explicit close, cancellation, and deadline controls.

use std::time::Duration;

use super::{lifecycle::Lifecycle, OwnedResource};

impl OwnedResource {
    /// Close the resource and release its host handle. Repeated closes succeed.
    ///
    /// # Examples
    ///
    /// ```
    /// use tetherscript::value::resource::OwnedResource;
    /// let mut channel = OwnedResource::channel(1)?;
    /// channel.close();
    /// assert!(channel.is_closed());
    /// # Ok::<(), String>(())
    /// ```
    pub fn close(&mut self) {
        self.payload.take();
        if self.lifecycle == Lifecycle::Open {
            self.lifecycle = Lifecycle::Closed;
        }
    }

    /// Cancel pending work, release the handle, and preserve cancelled state.
    ///
    /// # Errors
    ///
    /// Returns a handle-qualified error if host cancellation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use tetherscript::value::resource::OwnedResource;
    /// let mut timer = OwnedResource::timer(Duration::from_secs(1));
    /// timer.cancel()?;
    /// assert!(timer.is_cancelled());
    /// # Ok::<(), String>(())
    /// ```
    pub fn cancel(&mut self) -> Result<(), String> {
        if let Some(payload) = self.payload.as_mut() {
            payload.cancel()?;
        }
        self.payload.take();
        self.lifecycle = Lifecycle::Cancelled;
        Ok(())
    }

    /// Return whether explicit close released this handle.
    pub fn is_closed(&self) -> bool {
        self.lifecycle == Lifecycle::Closed
    }

    /// Return whether this resource was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.lifecycle == Lifecycle::Cancelled
    }

    /// Set a monotonic deadline relative to now.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use tetherscript::value::resource::OwnedResource;
    /// let mut task = OwnedResource::task();
    /// task.set_deadline_after(Duration::ZERO);
    /// assert!(task.is_expired());
    /// ```
    pub fn set_deadline_after(&mut self, duration: Duration) {
        self.deadline.set_after(duration);
    }

    /// Remove the current deadline.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use tetherscript::value::resource::OwnedResource;
    /// let mut task = OwnedResource::task();
    /// task.set_deadline_after(Duration::ZERO);
    /// task.clear_deadline();
    /// assert!(!task.is_expired());
    /// ```
    pub fn clear_deadline(&mut self) {
        self.deadline.clear();
    }

    /// Return whether the current monotonic deadline has elapsed.
    pub fn is_expired(&self) -> bool {
        self.deadline.expired()
    }
}
