const { controls } = require('./resource-method-controls');
const { operations } = require('./resource-method-operations');

const methods = {
  contains: ['value.contains(needle)', 'Return whether a string, list, or map contains a value.'],
  ends_with: ['text.ends_with(suffix)', 'Return whether a string ends with a suffix.'],
  err: ['result.err()', 'Return the Err message or nil.'],
  is_err: ['result.is_err()', 'Return true for Err results.'],
  is_ok: ['result.is_ok()', 'Return true for Ok results.'],
  join: ['list.join(separator)', 'Join list items into a string.'],
  keys: ['map.keys()', 'Return map keys as a list.'],
  len: ['value.len()', 'Return the length of a string, bytes, list, or map.'],
  lines: ['text.lines()', 'Split a string into lines.'],
  lower: ['text.lower()', 'Return a lowercase string.'],
  ok: ['result.ok()', 'Return the Ok value or nil.'],
  pop: ['list.pop()', 'Remove and return the last list item.'],
  push: ['list.push(value)', 'Append a value to a list.'],
  replace: ['text.replace(from, to)', 'Replace string occurrences.'],
  split: ['text.split(separator)', 'Split a string into a list.'],
  starts_with: ['text.starts_with(prefix)', 'Return whether a string starts with a prefix.'],
  trim: ['text.trim()', 'Trim surrounding whitespace.'],
  unwrap: ['result.unwrap()', 'Return Ok value or fail on Err.'],
  unwrap_or: ['result.unwrap_or(default)', 'Return Ok value or a default.'],
  upper: ['text.upper()', 'Return an uppercase string.'],
  values: ['map.values()', 'Return map values as a list.'],
  ...controls,
  ...operations,
};

module.exports = { methods };
