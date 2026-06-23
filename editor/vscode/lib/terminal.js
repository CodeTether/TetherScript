const path = require('path');

function quoted(value) {
  const text = String(value);
  if (process.platform === 'win32') return `'${text.replace(/'/g, "''")}'`;
  return `'${text.replace(/'/g, "'\\''")}'`;
}

function workspaceRoot(vscode) {
  const folders = vscode.workspace.workspaceFolders;
  return folders && folders.length ? folders[0].uri.fsPath : undefined;
}

function activeFile(vscode) {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage('Open a .tether file first.');
    return undefined;
  }

  const document = editor.document;
  if (document.isUntitled) {
    vscode.window.showErrorMessage('Save the .tether file first.');
    return undefined;
  }

  if (document.languageId !== 'tetherscript' && !document.fileName.endsWith('.tether')) {
    vscode.window.showErrorMessage('The active file is not a tetherscript file.');
    return undefined;
  }

  return document.fileName;
}

function commandLine(binary, file, args) {
  const suffix = args && args.trim().length ? ` ${args.trim()}` : '';
  if (process.platform === 'win32') return `& ${quoted(binary)}${suffix} ${quoted(file)}`;
  return `${quoted(binary)}${suffix} ${quoted(file)}`;
}

function agentFile(vscode) {
  const root = workspaceRoot(vscode);
  return root ? path.join(root, 'examples', 'agent_tui.tether') : undefined;
}

module.exports = { activeFile, agentFile, commandLine, workspaceRoot };
