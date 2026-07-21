const modulePath = require('./module-path');

function unique(module, declarations, text) {
  const used = new Set(declarations.map((item) => item.alias));
  for (const match of text.matchAll(/^\s*(?:async\s+)?fn\s+(\w+)|^\s*let(?:\s+mut)?\s+(\w+)/gm)) {
    used.add(match[1] || match[2]);
  }
  const base = modulePath.aliasFor(module.uri.fsPath);
  let alias = base;
  let suffix = 2;
  while (used.has(alias)) alias = `${base}_${suffix++}`;
  return alias;
}

module.exports = { unique };