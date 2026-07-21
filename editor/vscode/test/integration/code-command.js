const fs = require('fs');
const path = require('path');

function command() {
  if (process.env.VSCODE_CLI) return process.env.VSCODE_CLI;
  if (process.platform !== 'win32') return 'code';
  const installed = path.join(process.env.LOCALAPPDATA || '', 'Programs', 'Microsoft VS Code', 'Code.exe');
  return fs.existsSync(installed) ? installed : 'code';
}

module.exports = { command };