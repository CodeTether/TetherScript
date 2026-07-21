const core = {
  Ok: ['Ok(value)', 'Create a successful Result value.'],
  Err: ['Err(message)', 'Create an error Result value.'],
  assert: ['assert(condition[, message])', 'Fail execution when a condition is false.'],
  bytes: ['bytes(value)', 'Convert a string or byte list to bytes.'],
  eval: ['eval(source)', 'Evaluate source in the sandboxed tetherscript runtime.'],
  global_defined: ['global_defined(name)', 'Return whether a global binding exists.'],
  len: ['len(value)', 'Return the length of a string, bytes, list, or map.'],
  map: ['map()', 'Create an empty map.'],
  parse_float: ['parse_float(text)', 'Parse a float and return a Result.'],
  parse_int: ['parse_int(text)', 'Parse an integer and return a Result.'],
  print: ['print(...values)', 'Write values without a newline.'],
  println: ['println(...values)', 'Write values with a newline.'],
  str: ['str(value)', 'Convert a value to a string.'],
  type_of: ['type_of(value)', 'Return the runtime type name.'],
};

module.exports = { core };
