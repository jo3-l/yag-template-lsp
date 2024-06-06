import { resolve } from 'path';
import { ExtensionContext, window } from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient | undefined = undefined;

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export async function activate(_context: ExtensionContext) {
	try {
		await startClient();
	} catch (error) {
		void window.showErrorMessage(`Failed to activate yag-template-lsp: ${error}`);
		throw error;
	}
}

export function deactivate() {
	return client?.stop();
}

async function startClient() {
	const lspExecutable = {
		command: findLspExecutable(),
		options: { env: { ...process.env, RUST_BACKTRACE: '1' } },
	};

	const serverOptions: ServerOptions = {
		run: lspExecutable,
		debug: lspExecutable,
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'yagtemplate' }],
	};

	client = new LanguageClient('yag-template-lsp', 'YAGPDB Template Language Server', serverOptions, clientOptions);
	return client.start();
}

function findLspExecutable(): string {
	return resolve(__dirname, 'yag-template-lsp');
}
