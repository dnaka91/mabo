import { createStdioOptions, createUriConverters, startServer } from "@vscode/wasm-wasi-lsp";
import { type ProcessOptions, Wasm } from "@vscode/wasm-wasi/v1";
import { type ExtensionContext, Uri, commands, window, workspace } from "vscode";
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

export async function activate(context: ExtensionContext) {
  const wasm: Wasm = await Wasm.load();

  context.subscriptions.push(
    commands.registerCommand(Cmds.Restart, () => {
      client?.restart();
    }),
  );

  const channel = window.createOutputChannel("Mabo LSP Server");

  const serverOptions: ServerOptions = async () => {
    const options: ProcessOptions = {
      stdio: createStdioOptions(),
      mountPoints: [{ kind: "workspaceFolder" }],
    };
    const fileName = Uri.joinPath(context.extensionUri, "dist", "mabo-lsp.wasm");
    const buf = await workspace.fs.readFile(fileName);
    channel.appendLine(`read wasm file from ${fileName}`);
    const module = await WebAssembly.compile(buf);
    channel.appendLine("wasm compiled");
    const process = await wasm.createProcess(
      "lsp-server",
      module,
      { initial: 160, shared: true },
      options,
    );
    channel.appendLine("wasm process created");

    const decoder = new TextDecoder("utf-8");
    process.stderr?.onData((data) => {
      const msg = decoder.decode(data);
      channel.append(`SERVER >>> ${msg}`);
    });

    return startServer(process);
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "mabo" }],
    outputChannel: channel,
    uriConverters: createUriConverters(),
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher("**/*.mabo"),
    },
  };

  client = new LanguageClient("mabo", "Mabo Schema", serverOptions, clientOptions);

  try {
    channel.appendLine("starting lsp client");
    await client.start();
  } catch (error) {
    client.error("Start failed", error, "force");
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
