const factories = {
  channel: ['resource.channel(capacity)', 'Create a bounded owned channel.'],
  child_process: ['resource.child_process(command, arguments)', 'Spawn an owned child process.'],
  file: ['resource.file(path, mode)', 'Open an owned file handle.'],
  request_body: ['resource.request_body(body, capacity)', 'Create a bounded request-body reader.'],
  response_writer: ['resource.response_writer(capacity)', 'Create a bounded response writer.'],
  task: ['resource.task()', 'Create a pending cooperative task result.'],
  tcp_connect: ['resource.tcp_connect(host, port, timeout_ms)', 'Connect an owned nonblocking TCP stream.'],
  tcp_listen: ['resource.tcp_listen(host, port)', 'Bind an owned nonblocking TCP listener.'],
  timer: ['resource.timer(delay_ms)', 'Create an owned monotonic timer.'],
};

module.exports = { factories };
