const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;
let runTerminal;

function quoted(value) {
  const text = String(value);
  if (process.platform === 'win32') {
    return `'${text.replace(/'/g, "''")}'`;
  }
  return `'${text.replace(/'/g, "'\\''")}'`;
}

function commandLine(binary, file, extraArgs) {
  const args = extraArgs.trim();
  const suffix = args.length ? ` ${args}` : '';
  if (process.platform === 'win32') {
    return `& ${quoted(binary)} run${suffix} ${quoted(file)}`;
  }
  return `${quoted(binary)} run${suffix} ${quoted(file)}`;
}

function activeFile() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('Open a .tether file before running tetherscript.');
    return undefined;
  }

  const document = editor.document;
  if (document.isUntitled) {
    vscode.window.showErrorMessage('Save the .tether file before running it.');
    return undefined;
  }

  if (document.languageId !== 'tetherscript' && !document.fileName.endsWith('.tether')) {
    vscode.window.showErrorMessage('The active file is not a tetherscript file.');
    return undefined;
  }

  return document.fileName;
}

function runFile() {
  const file = activeFile();
  if (!file) return;

  const config = vscode.workspace.getConfiguration('tetherscript');
  const binary = config.get('serverPath') || 'tetherscript';
  const extraArgs = config.get('runArgs') || '';

  if (!runTerminal) {
    runTerminal = vscode.window.createTerminal({ name: 'tetherscript' });
  }
  runTerminal.show();
  runTerminal.sendText(commandLine(binary, file, extraArgs));
}

function activate(context) {
  const config = vscode.workspace.getConfiguration('tetherscript');
  const command = config.get('serverPath') || 'tetherscript';

  const serverOptions = {
    run:   { command, args: ['lsp'], transport: TransportKind.stdio },
    debug: { command, args: ['lsp'], transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'tetherscript' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{tether,kl}'),
    },
  };

  context.subscriptions.push(
    vscode.commands.registerCommand('tetherscript.runFile', runFile),
    vscode.window.onDidCloseTerminal((terminal) => {
      if (terminal === runTerminal) runTerminal = undefined;
    }),
  );

  client = new LanguageClient('tetherscript', 'tetherscript', serverOptions, clientOptions);
  client.start();
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
