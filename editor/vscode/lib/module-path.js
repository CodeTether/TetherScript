const fs = require('fs');
const path = require('path');

const RESERVED = new Set([
  'as', 'async', 'await', 'else', 'export', 'fn', 'for', 'if', 'import', 'in',
  'join', 'let', 'move', 'mut', 'panic', 'return', 'spawn', 'while', 'true', 'false', 'nil',
]);
function packageRoot(file) {
  let directory = path.dirname(file);
  while (true) {
    if (fs.existsSync(path.join(directory, 'tetherscript.json'))) return directory;
    const parent = path.dirname(directory);
    if (parent === directory) return path.dirname(file);
    directory = parent;
  }
}

function resolveImport(file, requested) {
  const relative = requested.startsWith('./') || requested.startsWith('../');
  if (!relative || path.extname(requested) !== '.tether' || path.isAbsolute(requested)) {
    return undefined;
  }
  return path.resolve(path.dirname(file), requested);
}

function contained(root, target) {
  const relative = path.relative(root, target);
  return relative === '' || (!relative.startsWith('..') && !path.isAbsolute(relative));
}

function importPath(fromFile, targetFile) {
  let relative = path.relative(path.dirname(fromFile), targetFile).replaceAll('\\', '/');
  if (!relative.startsWith('.')) relative = `./${relative}`;
  return relative;
}

function aliasFor(file) {
  const stem = path.basename(file, '.tether').replace(/[^A-Za-z0-9_]/g, '_');
  const safe = stem.replace(/^[^A-Za-z_]+/, '');
  if (!safe) return 'module';
  return RESERVED.has(safe) ? `${safe}_module` : safe;
}

function importInsertOffset(text) {
  const matches = [...text.matchAll(/^[ \t]*import\s+"[^"\r\n]+"\s+as\s+\w+[ \t]*\r?\n/gm)];
  if (matches.length === 0) return 0;
  const last = matches[matches.length - 1];
  return last.index + last[0].length;
}

module.exports = { aliasFor, contained, importInsertOffset, importPath, packageRoot, resolveImport };