# Owned runtime resources

The normal tetherscript runtime exposes a `resource` namespace. Its handles are
heap values, so `move handle` transfers ownership and tombstones the source
binding. Ordinary reads borrow the same lifecycle state. The sandboxed `eval`
runtime does not install these ambient host factories.

Every persistent boundary rejects borrowed resources and requires `move`. This
includes bindings, assignments, list/map storage, `Ok`, function returns,
`channel.send`, and `task.complete`. Validation follows resources nested inside
lists, maps, and results, and a rejected transfer leaves the original owner
live when the rejected value was borrowed. `channel.recv`, `list.pop`, and the
one-shot `task.result` transfer stored ownership back to the caller.

| Factory | Result type | Arguments |
|---|---|---|
| `resource.file` | `file` | path, mode (`read`, `write`, `append`, `read_write`) |
| `resource.child_process` | `child_process` | command, string argument list |
| `resource.child_process_bounded` | `child_process` | command, argument list, per-stream byte capacity |
| `resource.tcp_connect` | `tcp_stream` | host, port, connect timeout milliseconds |
| `resource.tcp_listen` | `tcp_listener` | host, port |
| `resource.request_body` | `request_body` | string/bytes, capacity |
| `resource.response_writer` | `response_writer` | capacity |
| `resource.task` | `task` | none |
| `resource.timer` | `timer` | delay milliseconds |
| `resource.channel` | `channel` | capacity |
| `resource.render_surface` | `render_surface` | width, height, scale, maximum pixels |

Every factory returns `Result`, and every recoverable I/O or readiness operation
does too. Shared controls are `close`, `cancel`, `is_closed`, `is_cancelled`,
`set_deadline`, `clear_deadline`, `deadline_remaining_ms`, and `is_expired`.
Closed, cancelled, and expired handles reject later operations with a
kind-and-method-qualified `Err`.

Resource-specific operations are:

- `file`: `read`, `write`, `flush`
- `child_process`: `id`, `try_wait`, `wait`, `kill`, `write_stdin`, `close_stdin`, `read_stdout`, `read_stderr`, `stdout_eof`, `stderr_eof`, `stream_capacity`
- `tcp_stream`: `read`, `write`, `peer_addr`, `shutdown`
- `tcp_listener`: `accept`, `local_addr`, `port`
- `request_body`: `read`, `remaining`, `capacity`
- `response_writer`: `write`, `body`, `len`, `capacity`
- `task`: `complete`, `result`, `is_complete`
- `timer`: `ready`, `remaining_ms`, `reset`
- `channel`: `send`, `recv`, `len`, `capacity`, `is_full`
- `render_surface`: `render`, `pixels`, `ppm`, `clear`, `has_frame`, `width`, `height`, `pixel_count`, `capacity`

TCP handles are nonblocking. `accept`, socket reads/writes, pending task results,
empty channel receives, full channel sends, and full response writes report
`backpressure` in their recoverable error. See
[`examples/owned_resources.tether`](../examples/owned_resources.tether) for a
cross-platform end-to-end example.

Child processes are supervised: all standard streams use bounded background
pumps, and closing or dropping the resource terminates and reaps a live child.
Stream reads and writes never wait on the operating-system pipe; they return a
recoverable `backpressure` error when no data or capacity is available. See
[`examples/process_streams.tether`](../examples/process_streams.tether).

Rendering surfaces hold at most one RGBA frame. Creation rejects dimensions
that exceed the explicit pixel capacity, while `clear` releases the frame
without closing the reusable surface. See
[`examples/render_surface.tether`](../examples/render_surface.tether).
