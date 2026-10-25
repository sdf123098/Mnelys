import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

// i18n must be initialized before the first render.
import './i18n';
import './styles/tokens.css';
import './styles/global.css';

import { App } from './App';
import { detectPlatform } from './ipc/window';
import { usePreferences } from './state/preferences-store';
import { applyAccent, applyTheme, resolveStoredTheme } from './state/theme';

// Apply the stored palette and accent before the first paint; the providers only keep
// them in sync afterwards. The platform lands on the root so the title bar can reserve
// the macOS traffic-light space without a layout shift.
document.documentElement.dataset['os'] = detectPlatform();
const storedPreferences = usePreferences.getState();
applyTheme(storedPreferences.theme);
applyAccent(storedPreferences.accent, resolveStoredTheme(storedPreferences.theme));

const container = document.getElementById('root');

if (container === null) {
  throw new Error('index.html is missing the #root container; the webview cannot mount Mnelys.');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
