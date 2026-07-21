const controls = {
  cancel: ['resource.cancel()', 'Cancel work, release the handle, and return a Result.'],
  clear_deadline: ['resource.clear_deadline()', 'Remove the resource deadline.'],
  close: ['resource.close()', 'Release the owned handle and return a Result.'],
  deadline_remaining_ms: ['resource.deadline_remaining_ms()', 'Return remaining deadline time or nil.'],
  is_cancelled: ['resource.is_cancelled()', 'Return whether the resource was cancelled.'],
  is_closed: ['resource.is_closed()', 'Return whether the resource was closed.'],
  is_expired: ['resource.is_expired()', 'Return whether the resource deadline elapsed.'],
  kind: ['resource.kind()', 'Return the resource kind name.'],
  set_deadline: ['resource.set_deadline(delay_ms)', 'Set a monotonic resource deadline.'],
};

module.exports = { controls };
