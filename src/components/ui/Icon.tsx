import type { LucideIcon, LucideProps } from 'lucide-react';
import {
  ArrowRight,
  BookOpen,
  Box,
  Boxes,
  Bug,
  Check,
  ChevronRight,
  CircleAlert,
  CircleCheck,
  CircleUserRound,
  CodeXml,
  Coffee,
  Compass,
  Copy,
  Cpu,
  Database,
  Download,
  ExternalLink,
  FileText,
  FolderOpen,
  Gamepad2,
  Globe,
  HardDrive,
  House,
  Info,
  Languages,
  Layers,
  LibraryBig,
  ListChecks,
  LoaderCircle,
  LogOut,
  MemoryStick,
  Minus,
  Monitor,
  Moon,
  Network,
  Package,
  Palette,
  Play,
  Plus,
  RefreshCw,
  Rocket,
  Server,
  Settings2,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Square,
  SquareTerminal,
  Stethoscope,
  Sun,
  Trash2,
  TriangleAlert,
  Users,
  Wrench,
  X,
} from 'lucide-react';

/**
 * The only icon set in the interface (see docs/ICONS.md).
 *
 * Every glyph is Lucide (ISC), rendered as an SVG component and bundled locally, so
 * there is no icon font, no runtime download and no license-ambiguous pictogram.
 * Components use semantic names here rather than importing Lucide directly; that
 * keeps one stroke weight across the shell and makes a future set swap a one-file
 * change. The application/brand icon is source artwork and is not part of this set.
 */
const iconRegistry = {
  start: House,
  instances: Boxes,
  downloads: Download,
  multiplayer: Users,
  developer: Wrench,
  accounts: CircleUserRound,
  settings: Settings2,
  tasks: ListChecks,

  play: Play,
  plus: Plus,
  close: X,
  minimize: Minus,
  maximize: Square,
  restore: Copy,
  chevronRight: ChevronRight,
  arrowRight: ArrowRight,
  check: Check,
  warning: TriangleAlert,
  info: Info,
  loading: LoaderCircle,
  refresh: RefreshCw,
  external: ExternalLink,
  filter: SlidersHorizontal,
  trash: Trash2,

  theme: Palette,
  language: Languages,
  sun: Sun,
  moon: Moon,
  monitor: Monitor,
  game: Gamepad2,
  java: Coffee,
  shield: ShieldCheck,
  network: Network,
  globe: Globe,
  server: Server,
  database: Database,
  storage: HardDrive,
  cpu: Cpu,
  memory: MemoryStick,
  layers: Layers,
  package: Package,
  folder: FolderOpen,
  library: LibraryBig,
  compass: Compass,
  book: BookOpen,
  file: FileText,
  box: Box,
  terminal: SquareTerminal,
  code: CodeXml,
  bug: Bug,
  diagnosis: Stethoscope,
  logout: LogOut,
  sparkles: Sparkles,
  rocket: Rocket,

  statusOk: CircleCheck,
  statusError: CircleAlert,
} as const satisfies Record<string, LucideIcon>;

export type IconName = keyof typeof iconRegistry;

export interface IconProps extends Omit<LucideProps, 'ref'> {
  readonly name: IconName;
  /** Supply only when the icon carries meaning no adjacent text already conveys. */
  readonly label?: string;
}

const DEFAULT_SIZE = 18;
const DEFAULT_STROKE_WIDTH = 1.75;

export function Icon({
  name,
  label,
  size = DEFAULT_SIZE,
  strokeWidth = DEFAULT_STROKE_WIDTH,
  ...rest
}: IconProps) {
  const Glyph = iconRegistry[name];

  return (
    <Glyph
      size={size}
      strokeWidth={strokeWidth}
      focusable="false"
      aria-hidden={label === undefined ? true : undefined}
      aria-label={label}
      role={label === undefined ? undefined : 'img'}
      {...rest}
    />
  );
}
