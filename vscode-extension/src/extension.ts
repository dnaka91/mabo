import { ExtensionContext, commands, workspace } from "vscode";
import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

enum Cmds {
  Restart = "stef.restart",
}

export function activate(context: ExtensionContext) {
  context.subscriptions.push(
    commands.registerCommand(Cmds.Restart, () => {
      client?.restart();
    }),
  );

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
    "stef",
    "Strongly Typed Encoding Format",
    serverOptions,
    clientOptions,
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
