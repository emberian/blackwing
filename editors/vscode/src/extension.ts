import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // Get the server path from configuration, or use default
    const config = workspace.getConfiguration('holdsmith');
    let serverPath = config.get<string>('serverPath') || '';

    if (!serverPath) {
        // Try to find holdsmith-lsp in common locations
        // In development, it's in the target directory
        const workspaceFolder = workspace.workspaceFolders?.[0];
        if (workspaceFolder) {
            // Check if we're in the blackwing workspace
            const devPath = path.join(
                workspaceFolder.uri.fsPath,
                'target',
                'debug',
                'holdsmith-lsp'
            );
            serverPath = devPath;
        }

        // Fallback to expecting it in PATH
        if (!serverPath) {
            serverPath = 'holdsmith-lsp';
        }
    }

    const serverExecutable: Executable = {
        command: serverPath,
        args: [],
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        debug: serverExecutable,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'scene' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.scene'),
        },
    };

    client = new LanguageClient(
        'holdsmith',
        'Holdsmith Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
