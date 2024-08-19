import { resolve } from 'path';
import { ExtensionContext, window, workspace, WorkspaceConfiguration } from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient | undefined = undefined;

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export async function activate(_context: ExtensionContext) {
	const config = workspace.getConfiguration('yag-template-lsp');
	try {
		await startClient(config);
	} catch (error) {
		void window.showErrorMessage(`Failed to activate yag-template-lsp: ${error}`);
		throw error;
	}
}

export function deactivate() {
	return client?.stop();
}

async function startClient(config: WorkspaceConfiguration) {
	const extraEnv = config.get<Record<string, string> | null>('server.extraEnv') ?? {};
	const run = {
		command: getLanguageServerBinary(config),
		options: { env: { ...process.env, ...extraEnv, RUST_BACKTRACE: '1' } },
	};

	const serverOptions: ServerOptions = {
		run,
		debug: run,
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'yag' }],
		initializationOptions: config,
	};

	client = new LanguageClient('yag-template-lsp', 'YAGPDB Template Language Server', serverOptions, clientOptions);
	return client.start();
}

function getLanguageServerBinary(config: WorkspaceConfiguration) {
	const localServerPath = config.get<string | null>('serverPath');
	return localServerPath || bundledLanguageServer();
}

function bundledLanguageServer() {
	if (process.platform === 'win32') return resolve(__dirname, 'yag-template-lsp.exe');
	return resolve(__dirname, 'yag-template-lsp');
}
