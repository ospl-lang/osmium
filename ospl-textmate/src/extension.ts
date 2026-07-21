import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient;

function startLsp() {
    const serverOptions: ServerOptions = {
        command: "/opt/ospl/ospl-lsp",
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
}

function stopLsp() {
    client?.stop();
}

export function activate(context: vscode.ExtensionContext) {
    context.subscriptions.push(client);
    vscode.commands.registerCommand(
        "ospl.startLsp",
        () => {
            startLsp()
        }
    );

    vscode.commands.registerCommand(
        "ospl.stopLsp",
        () => {
            stopLsp()
        }
    );
}

export function deactivate() {
    return client?.stop();
}