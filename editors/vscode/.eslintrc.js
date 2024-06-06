const { resolve } = require('node:path');

module.exports = {
	root: true,
	parser: '@typescript-eslint/parser',
	parserOptions: {
		ecmaVersion: 6,
		sourceType: 'module',
		project: resolve(__dirname, 'tsconfig.json'),
	},
	plugins: ['@typescript-eslint'],
	extends: ['plugin:@typescript-eslint/recommended-type-checked', 'plugin:prettier/recommended'],
	ignorePatterns: ['out', 'dist', '**/*.d.ts', 'esbuild.js', '.eslintrc.js'],
	rules: {
		'@typescript-eslint/restrict-template-expressions': 'off',
	},
};
