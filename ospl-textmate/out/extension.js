"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const serverOptions = {
        command: "ospl-lsp"
    };
    const clientOptions = {
        documentSelector: [
            {
                scheme: "file",
                language: "ospl"
            }
        ]
    };
    client = new node_1.LanguageClient("ospl", "OSPL Language Server", serverOptions, clientOptions);
    client.start();
    context.subscriptions.push(client);
}
function deactivate() {
    return client?.stop();
}
//# sourceMappingURL=extension.js.map