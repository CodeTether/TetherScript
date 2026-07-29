//! Agent subprocess cleanup.

use super::AgentClient;

impl Drop for AgentClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
