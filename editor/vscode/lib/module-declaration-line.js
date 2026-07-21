const IDENT = '[A-Za-z_][A-Za-z0-9_]*';

function parse(line, start) {
  const fn = line.match(new RegExp(`^\\s*(?:async\\s+)?fn\\s+(${IDENT})\\s*\\(([^)]*)\\)`));
  if (fn) return {
    name: fn[1],
    kind: 'function',
    signature: `${fn[1]}(${fn[2]})`,
    start: start + fn[0].lastIndexOf(fn[1]),
  };
  const value = line.match(new RegExp(`^\\s*let(?:\\s+mut)?\\s+(${IDENT})`));
  if (!value) return undefined;
  return {
    name: value[1],
    kind: 'value',
    signature: value[1],
    start: start + line.lastIndexOf(value[1]),
  };
}

module.exports = { parse };