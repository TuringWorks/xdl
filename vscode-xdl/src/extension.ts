import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window, OutputChannel } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: OutputChannel;

export async function activate(context: ExtensionContext) {
    outputChannel = window.createOutputChannel('XDL Language Server');
    outputChannel.appendLine('XDL extension activating...');

    const config = workspace.getConfiguration('xdl');
    const lspEnabled = config.get<boolean>('lsp.enabled', true);

    if (!lspEnabled) {
        outputChannel.appendLine('XDL LSP is disabled in settings');
        return;
    }

    const serverPath = await getServerPath(context, config);
    if (!serverPath) {
        window.showErrorMessage(
            'Could not find xdl-lsp executable. Please configure xdl.lsp.path or ensure it is in your PATH.'
        );
        return;
    }

    outputChannel.appendLine(`Using XDL LSP server at: ${serverPath}`);

    const serverOptions: ServerOptions = {
        run: { command: serverPath } as Executable,
        debug: { command: serverPath } as Executable,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'xdl' },
        ],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.{xdl,pro}'),
        },
        outputChannel,
        traceOutputChannel: outputChannel,
    };

    client = new LanguageClient(
        'xdl-lsp',
        'XDL Language Server',
        serverOptions,
        clientOptions
    );

    // Register commands
    context.subscriptions.push(
        {
            dispose: () => {
                if (client) {
                    client.stop();
                }
            },
        }
    );

    // Restart server command
    const restartCommand = 'xdl.restartServer';
    context.subscriptions.push(
        require('vscode').commands.registerCommand(restartCommand, async () => {
            outputChannel.appendLine('Restarting XDL Language Server...');
            if (client) {
                await client.stop();
                await client.start();
                outputChannel.appendLine('XDL Language Server restarted');
            }
        })
    );

    // Run file command
    const runFileCommand = 'xdl.runFile';
    context.subscriptions.push(
        require('vscode').commands.registerCommand(runFileCommand, async () => {
            const editor = window.activeTextEditor;
            if (!editor) {
                window.showErrorMessage('No active editor');
                return;
            }

            const filePath = editor.document.uri.fsPath;
            if (!filePath.endsWith('.xdl') && !filePath.endsWith('.pro')) {
                window.showErrorMessage('Current file is not an XDL file');
                return;
            }

            // Save the file first
            await editor.document.save();

            // Create terminal and run
            const terminal = window.createTerminal('XDL');
            terminal.show();
            terminal.sendText(`xdl "${filePath}"`);
        })
    );

    // Start the client
    try {
        await client.start();
        outputChannel.appendLine('XDL Language Server started successfully');
    } catch (error) {
        outputChannel.appendLine(`Failed to start XDL Language Server: ${error}`);
        window.showErrorMessage(`Failed to start XDL Language Server: ${error}`);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

async function getServerPath(
    context: ExtensionContext,
    config: ReturnType<typeof workspace.getConfiguration>
): Promise<string | undefined> {
    // First, check user-configured path
    const configuredPath = config.get<string>('lsp.path');
    if (configuredPath && configuredPath.trim() !== '') {
        if (fs.existsSync(configuredPath)) {
            return configuredPath;
        }
        outputChannel.appendLine(`Configured LSP path not found: ${configuredPath}`);
    }

    // Check bundled binary
    const bundledPath = getBundledServerPath(context);
    if (bundledPath && fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // Check PATH
    const pathServer = await findInPath('xdl-lsp');
    if (pathServer) {
        return pathServer;
    }

    return undefined;
}

function getBundledServerPath(context: ExtensionContext): string | undefined {
    const platform = process.platform;
    const arch = process.arch;

    let binaryName = 'xdl-lsp';
    if (platform === 'win32') {
        binaryName = 'xdl-lsp.exe';
    }

    // Look for platform-specific binary
    const candidates = [
        path.join(context.extensionPath, 'bin', `${platform}-${arch}`, binaryName),
        path.join(context.extensionPath, 'bin', binaryName),
        path.join(context.extensionPath, 'server', binaryName),
    ];

    for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }

    return undefined;
}

async function findInPath(executable: string): Promise<string | undefined> {
    const isWindows = process.platform === 'win32';
    const pathSeparator = isWindows ? ';' : ':';
    const pathEnv = process.env.PATH || '';
    const extensions = isWindows ? ['.exe', '.cmd', '.bat', ''] : [''];

    for (const dir of pathEnv.split(pathSeparator)) {
        for (const ext of extensions) {
            const fullPath = path.join(dir, executable + ext);
            if (fs.existsSync(fullPath)) {
                return fullPath;
            }
        }
    }

    return undefined;
}
