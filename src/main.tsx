import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

// i18n must be initialized before the first render.
import './i18n';
import './styles/tokens.css';
import './styles/global.css';

import { App } from './App';
import { usePreferences } from './state/preferences-store';
import { applyTheme } from './state/theme';

// Apply the stored palette before the first paint; the provider only keeps it in sync.
applyTheme(usePreferences.getState().theme);

const container = document.getElementById('root');

if (container === null) {
  throw new Error('index.html is missing the #root container; the webview cannot mount Mnelys.');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
