import { type ExtensionContext, commands, workspace } from "vscode";
import {
  type Executable,
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

enum Cmds {
  Restart = "mabo.restart",
}

export function activate(context: ExtensionContext) {
  context.subscriptions.push(
    commands.registerCommand(Cmds.Restart, () => {
      client?.restart();
    }),
  );

  const executable: Executable = {
    command: "mabo-lsp",
    transport: TransportKind.stdio,
  };

  const serverOptions: ServerOptions = {
    run: executable,
    debug: executable,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "mabo" }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.mabo"),
    },
  };

  client = new LanguageClient("mabo", "Mabo Schema", serverOptions, clientOptions);

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
