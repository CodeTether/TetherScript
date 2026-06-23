const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

function createClient(vscode) {
  const config = vscode.workspace.getConfiguration('tetherscript');
  const command = config.get('serverPath') || 'tetherscript';
  const serverOptions = {
    run: { command, args: ['lsp'], transport: TransportKind.stdio },
    debug: { command, args: ['lsp'], transport: TransportKind.stdio },
  };
  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'tetherscript' }],
    synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{tether,kl}') },
  };
  return new LanguageClient('tetherscript', 'tetherscript', serverOptions, clientOptions);
}

module.exports = { createClient };
