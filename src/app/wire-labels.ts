import type { IconName } from '@/components/ui/Icon';
import type { ContentKind, LoaderKind, VersionEntry } from '@/ipc/commands';

/** The order loaders are offered in, from the simplest to the most involved install. */
export const LOADER_ORDER: readonly LoaderKind[] = [
  'vanilla',
  'fabric',
  'quilt',
  'legacy-fabric',
  'forge',
  'neoforge',
];

export const LOADER_KEYS = {
  vanilla: 'loader.vanilla',
  fabric: 'loader.fabric',
  quilt: 'loader.quilt',
  'legacy-fabric': 'loader.legacyFabric',
  forge: 'loader.forge',
  neoforge: 'loader.neoforge',
} as const satisfies Readonly<Record<LoaderKind, string>>;

export const CONTENT_KIND_ORDER: readonly ContentKind[] = [
  'mod',
  'resourcepack',
  'shader',
  'world',
  'datapack',
];

export const CONTENT_KIND_KEYS = {
  mod: 'contentKind.mod',
  resourcepack: 'contentKind.resourcepack',
  shader: 'contentKind.shader',
  world: 'contentKind.world',
  datapack: 'contentKind.datapack',
} as const satisfies Readonly<Record<ContentKind, string>>;

export const CONTENT_KIND_ICONS = {
  mod: 'package',
  resourcepack: 'layers',
  shader: 'sparkles',
  world: 'globe',
  datapack: 'database',
} as const satisfies Readonly<Record<ContentKind, IconName>>;

export const VERSION_TYPE_KEYS = {
  release: 'version.type.release',
  snapshot: 'version.type.snapshot',
  old_beta: 'version.type.oldBeta',
  old_alpha: 'version.type.oldAlpha',
} as const satisfies Readonly<Record<VersionEntry['type'], string>>;
