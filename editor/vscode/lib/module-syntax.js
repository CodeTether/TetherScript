const { declarations, topLevelLines } = require('./module-declarations');

const IDENT = '[A-Za-z_][A-Za-z0-9_]*';

function imports(text) {
  const found = [];
  const pattern = new RegExp(`^\\s*import\\s+"([^"\\r\\n]+)"\\s+as\\s+(${IDENT})`);
  for (const { line, start } of topLevelLines(text)) {
    const match = line.match(pattern);
    if (!match) continue;
    found.push({
      path: match[1], alias: match[2], start,
      pathStart: start + match[0].indexOf(match[1]),
      aliasStart: start + match[0].lastIndexOf(match[2]),
    });
  }
  return found;
}

function exported(text) {
  const declared = declarations(text);
  const pattern = new RegExp(`^\\s*export\\s+(${IDENT})`);
  const found = [];
  for (const { line, start } of topLevelLines(text)) {
    const match = line.match(pattern);
    if (!match) continue;
    const name = match[1];
    found.push(declared.get(name) || {
      name, kind: 'unknown', signature: name,
      start: start + match[0].lastIndexOf(name),
    });
  }
  return found;
}

function memberBefore(text) {
  const match = text.match(new RegExp(`(${IDENT})\\.(${IDENT})?$`));
  return match ? { alias: match[1], prefix: match[2] || '' } : undefined;
}

module.exports = { declarations, exported, imports, memberBefore };