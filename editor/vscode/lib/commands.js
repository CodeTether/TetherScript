const childProcess = require('child_process');
const fs = require('fs');
const { activeFile, agentFile, commandLine, workspaceRoot } = require('./terminal');

let runTerminal;
let agentTerminal;

function config(vscode) {
  return vscode.workspace.getConfiguration('tetherscript');
}

function binary(vscode) {
  return config(vscode).get('serverPath') || 'tetherscript';
}

function runFile(vscode) {
  const file = activeFile(vscode);
  if (!file) return;

  if (!runTerminal) runTerminal = vscode.window.createTerminal({ name: 'tetherscript' });
  runTerminal.show();
  runTerminal.sendText(commandLine(binary(vscode), file, `run ${config(vscode).get('runArgs') || ''}`));
}

function runAgentTui(vscode) {
  const file = agentFile(vscode);
  if (!file || !fs.existsSync(file)) {
    vscode.window.showErrorMessage('Could not find examples/agent_tui.tether in the current workspace.');
    return;
  }

  if (!agentTerminal) agentTerminal = vscode.window.createTerminal({ name: 'tetherscript agent' });
  agentTerminal.show();
  agentTerminal.sendText(commandLine(binary(vscode), file, 'run --access-mode full'));
}

function outputName(kind) {
  return `tetherscript ${kind}`;
}

function inspect(vscode, kind) {
  const file = activeFile(vscode);
  if (!file) return;

  const channel = vscode.window.createOutputChannel(outputName(kind));
  const flag = `--${kind}`;
  const cwd = workspaceRoot(vscode) || process.cwd();
  channel.clear();
  channel.show(true);
  channel.appendLine(`> ${binary(vscode)} inspect ${flag} ${file}`);

  childProcess.execFile(binary(vscode), ['inspect', flag, file], { cwd }, (err, stdout, stderr) => {
    if (stdout) channel.append(stdout);
    if (stderr) channel.append(stderr);
    if (err) channel.appendLine(`\nexit ${err.code || 1}`);
  });
}

function registerCommands(vscode, context) {
  context.subscriptions.push(
    vscode.commands.registerCommand('tetherscript.runFile', () => runFile(vscode)),
    vscode.commands.registerCommand('tetherscript.runAgentTui', () => runAgentTui(vscode)),
    vscode.commands.registerCommand('tetherscript.inspectTokens', () => inspect(vscode, 'tokens')),
    vscode.commands.registerCommand('tetherscript.inspectAst', () => inspect(vscode, 'ast')),
    vscode.commands.registerCommand('tetherscript.inspectBytecode', () => inspect(vscode, 'bytecode')),
    vscode.window.onDidCloseTerminal((terminal) => {
      if (terminal === runTerminal) runTerminal = undefined;
      if (terminal === agentTerminal) agentTerminal = undefined;
    }),
  );
}

module.exports = { registerCommands };
