import js from '@eslint/js';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: ['dist', 'node_modules', 'src-tauri/target', 'src-tauri/gen', 'coverage'],
  },
  js.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    // The type-aware configs only apply where the parser can reach a tsconfig.
    extends: [...tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      ecmaVersion: 2023,
      globals: globals.browser,
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': 'error',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      // ADR 0006: every Tauri command must go through src/ipc/client.ts.
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tauri-apps/api/core',
              importNames: ['invoke'],
              message:
                'Call the Rust core through src/ipc/client.ts, never through invoke() directly (ADR 0006).',
            },
          ],
          patterns: [
            {
              group: ['@tauri-apps/plugin-*'],
              message:
                'Tauri plugins are reached through src/ipc/client.ts so the capability surface stays reviewable (ADR 0005, ADR 0006).',
            },
          ],
        },
      ],
    },
  },
  {
    files: ['vite.config.ts', 'eslint.config.js'],
    languageOptions: {
      globals: globals.node,
    },
  },
  {
    // The single module allowed to touch the Tauri API surface. Everything else must
    // reach the core through it, which is what makes the capability surface reviewable.
    files: ['src/ipc/client.ts'],
    rules: {
      'no-restricted-imports': 'off',
    },
  },
);
