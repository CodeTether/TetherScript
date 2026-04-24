const { workspace } = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
  const config = workspace.getConfiguration('tetherscript');
  const command = config.get('serverPath') || 'tetherscript';

  const serverOptions = {
    run:   { command, args: ['--lsp'], transport: TransportKind.stdio },
    debug: { command, args: ['--lsp'], transport: TransportKind.stdio },
  };

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'tetherscript' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.{tether,kl}'),
    },
  };

  client = new LanguageClient('tetherscript', 'TetherScript', serverOptions, clientOptions);
  client.start();
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
