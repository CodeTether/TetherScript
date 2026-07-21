const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const code = require('./code-command');

const extension = path.resolve(__dirname, '../..');
const root = path.resolve(extension, '../..');
const fixture = path.join(root, '.tmp', 'vscode-module-live');
const stamp = Date.now();
const userData = path.join(root, '.tmp', `vscode-test-user-${stamp}`);
const extensions = path.join(root, '.tmp', `vscode-test-extensions-${stamp}`);
const binaryName = process.platform === 'win32' ? 'tetherscript.exe' : 'tetherscript';
const candidates = [
  path.join(root, '.tmp', 'package-module-release-target', 'release', binaryName),
  path.join(root, 'target', 'debug', binaryName),
];
const server = candidates.find((file) => fs.existsSync(file)) || 'tetherscript';

fs.mkdirSync(path.join(fixture, '.vscode'), { recursive: true });
fs.writeFileSync(path.join(fixture, 'tetherscript.json'), JSON.stringify({
  schema: 1,
  package: { name: 'vscode-live', version: '0.1.0', entry: 'main.tether' },
}, null, 2));
fs.writeFileSync(path.join(fixture, 'math.tether'), [
  'fn add(left, right) {',
  '    left + right',
  '}',
  'let answer = 42',
  'export add',
  'export answer',
].join('\n'));
fs.writeFileSync(path.join(fixture, '.vscode', 'settings.json'), JSON.stringify({
  'tetherscript.serverPath': server.replaceAll('\\', '/'),
}, null, 2));

const args = [
  '--disable-workspace-trust', '--skip-welcome', '--skip-release-notes',
  `--user-data-dir=${userData}`, `--extensions-dir=${extensions}`,
  `--extensionDevelopmentPath=${extension}`,
  `--extensionTestsPath=${path.join(__dirname, 'run.js')}`,
  fixture,
];
const result = spawnSync(code.command(), args, { stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status === 0 ? 0 : (result.status || 1);