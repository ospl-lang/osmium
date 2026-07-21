import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const serverOptions: ServerOptions = {
        command: "/home/colton/.local/bin/ospl-lsp"
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            {
                scheme: "file",
                language: "ospl"
            }
        ]
    };

    client = new LanguageClient(
        "ospl",
        "OSPL Language Server",
        serverOptions,
        clientOptions
    );

    client.start();

    context.subscriptions.push(client);
}

export function deactivate() {
    return client?.stop();
}