//! Child termination, reaping, and pump shutdown.

use super::{status, Handle};

impl Handle {
    pub(in crate::value::resource) fn cancel(&mut self) -> Result<(), String> {
        self.stdin.close();
        self.stdout.close();
        self.stderr.close();
        if self.child.try_wait().map_err(status::wait_error)?.is_none() {
            self.child.kill().map_err(status::kill_error)?;
            self.child.wait().map_err(status::wait_error)?;
        }
        Ok(())
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = self.cancel();
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
