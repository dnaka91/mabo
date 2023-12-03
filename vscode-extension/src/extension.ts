import { ExtensionContext, workspace } from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const executable: Executable = {
    command: "stef-lsp",
    transport: TransportKind.stdio,
  };

  const serverOptions: ServerOptions = {
    run: executable,
    debug: {
      ...executable,
      options: {
        env: {
          RUST_LOG: "info,tower_lsp=trace,stef_lsp=trace",
        },
      },
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "stef" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.stef"),
    },
  };

  client = new LanguageClient(
    "stef-lsp",
    "STEF Language Server",
    serverOptions,
    clientOptions,
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
