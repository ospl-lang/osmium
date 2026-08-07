killall ospl-lsp -9

set -e
cargo build --release -p ospl-lsp
cp ../../target/release/ospl-lsp /opt/ospl/ospl-lsp

npm run compile
npx @vscode/vsce package --allow-missing-repository
code --install-extension ospl-textmate-0.0.1.vsix
