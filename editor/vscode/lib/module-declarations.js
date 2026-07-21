const declaration = require('./module-declaration-line');
const { codeOnly } = require('./source-mask');

function topLevelLines(text) {
  const code = codeOnly(text);
  const lines = [];
  let depth = 0;
  for (const match of code.matchAll(/[^\n]*(?:\n|$)/g)) {
    if (!match[0]) continue;
    const codeLine = match[0].replace(/\r?\n$/, '');
    const sourceLine = text.slice(match.index, match.index + match[0].length)
      .replace(/\r?\n$/, '');
    if (depth === 0) lines.push({ code: codeLine, line: sourceLine, start: match.index });
    for (const char of codeLine) {
      if (char === '{') depth += 1;
      if (char === '}') depth = Math.max(0, depth - 1);
    }
  }
  return lines;
}

function declarations(text) {
  const found = new Map();
  for (const { code, start } of topLevelLines(text)) {
    const item = declaration.parse(code, start);
    if (item) found.set(item.name, item);
  }
  return found;
}

module.exports = { declarations, topLevelLines };