import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

// i18n must be initialized before the first render.
import './i18n';
import './styles/tokens.css';
import './styles/global.css';

import { App } from './App';

const container = document.getElementById('root');

if (container === null) {
  throw new Error('index.html is missing the #root container; the webview cannot mount Mnelys.');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
