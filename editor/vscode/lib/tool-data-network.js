const network = {
  http_get: ['http_get(url)', 'Run a blocking HTTP GET request.'],
  http_head: ['http_head(url)', 'Run a blocking HTTP HEAD request.'],
  http_post: ['http_post(url, body)', 'Run a blocking HTTP POST request.'],
  http_request: ['http_request(method, url[, body[, headers]])', 'Run a blocking HTTP request.'],
  http_serve: ['http_serve(port, handler)', 'Serve HTTP requests with a script handler.'],
  http_serve_static: ['http_serve_static(port, root_dir)', 'Serve files beneath a directory.'],
  https_serve: ['https_serve(port, certificate_pem, private_key_pem, handler)', 'Serve HTTPS with a PEM identity.'],
  smtp_send: ['smtp_send(host, port, from, to, subject, body)', 'Send an email over SMTP.'],
};

module.exports = { network };
