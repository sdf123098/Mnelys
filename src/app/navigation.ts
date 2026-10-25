import type { IconName } from '@/components/ui/Icon';

/**
 * The primary information architecture from the product plan: Start, Instances,
 * Downloads, Multiplayer, Developer (mode-gated), Settings. Accounts is a destination
 * reachable from the top bar, not a primary rail item.
 */
export const SECTIONS = [
  {
    id: 'start',
    to: '/start',
    labelKey: 'nav.start',
    icon: 'start',
    inNav: true,
    developerOnly: false,
  },
  {
    id: 'instances',
    to: '/instances',
    labelKey: 'nav.instances',
    icon: 'instances',
    inNav: true,
    developerOnly: false,
  },
  {
    id: 'downloads',
    to: '/downloads',
    labelKey: 'nav.downloads',
    icon: 'downloads',
    inNav: true,
    developerOnly: false,
  },
  {
    id: 'multiplayer',
    to: '/multiplayer',
    labelKey: 'nav.multiplayer',
    icon: 'multiplayer',
    inNav: true,
    developerOnly: false,
  },
  {
    id: 'developer',
    to: '/developer',
    labelKey: 'nav.developer',
    icon: 'developer',
    inNav: true,
    developerOnly: true,
  },
  {
    id: 'accounts',
    to: '/accounts',
    labelKey: 'nav.accounts',
    icon: 'accounts',
    inNav: false,
    developerOnly: false,
  },
  {
    id: 'settings',
    to: '/settings',
    labelKey: 'nav.settings',
    icon: 'settings',
    inNav: true,
    developerOnly: false,
  },
] as const satisfies ReadonlyArray<{
  readonly id: string;
  readonly to: string;
  readonly labelKey: string;
  readonly icon: IconName;
  readonly inNav: boolean;
  readonly developerOnly: boolean;
}>;

export type Section = (typeof SECTIONS)[number];

export function findSection(pathname: string): Section | undefined {
  return SECTIONS.find(
    (section) => pathname === section.to || pathname.startsWith(`${section.to}/`),
  );
}
