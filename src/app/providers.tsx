import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { isRtlLanguage } from '@/i18n/resources';
import { usePreferences } from '@/state/preferences-store';
import { applyAccent, applyTheme, useResolvedTheme } from '@/state/theme';

function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // The Rust core owns the data; this cache is disposable. A short stale time keeps
        // the UI responsive without pretending the frontend is the source of truth.
        staleTime: 5_000,
        gcTime: 5 * 60_000,
        retry: 1,
        refetchOnWindowFocus: false,
      },
      mutations: {
        retry: 0,
      },
    },
  });
}

export function AppProviders({ children }: { readonly children: ReactNode }) {
  const [queryClient] = useState(createQueryClient);
  const theme = usePreferences((state) => state.theme);
  const accent = usePreferences((state) => state.accent);
  const resolvedTheme = useResolvedTheme(theme);
  const { i18n } = useTranslation();

  useEffect(() => {
    applyTheme(theme);
    applyAccent(accent, resolvedTheme);
  }, [theme, accent, resolvedTheme]);

  useEffect(() => {
    const language = i18n.resolvedLanguage ?? 'en';
    document.documentElement.lang = language;
    document.documentElement.dir = isRtlLanguage(language) ? 'rtl' : 'ltr';
  }, [i18n, i18n.resolvedLanguage]);

  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
}
