const data = {
  base64_decode: ['base64_decode(text)', 'Decode Base64 text.'],
  base64_encode: ['base64_encode(text)', 'Encode text as Base64.'],
  json_encode: ['json_encode(value)', 'Encode a value as compact JSON.'],
  json_encode_pretty: ['json_encode_pretty(value)', 'Encode a value as formatted JSON.'],
  json_parse: ['json_parse(text)', 'Parse JSON into tetherscript values.'],
  sha256_hex: ['sha256_hex(text)', 'Compute a SHA-256 hex digest.'],
  tera_render: ['tera_render(template, context[, escape])', 'Render an optional Tera template with map data.'],
  url_parse: ['url_parse(url)', 'Parse a URL into a map.'],
};

module.exports = { data };
