<p align="center">
  <img src="assets/branding/mnelys-icon-source.jpg" width="220" alt="Mnelys アイコン">
</p>

<h1 align="center">Mnelys</h1>

<p align="center">
  プレイヤーと Mod 開発者のための AI ネイティブ Minecraft: Java Edition ランチャー／開発ワークスペース
</p>

<p align="center">
  <a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · 日本語
</p>

> [!IMPORTANT]
> Mnelys の技術基盤は **Tauri v2 + React 19** です（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md) と [ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md) を参照）。現在の `main` は M1–M6 のコア機能（アカウント、インスタンス、Java、メタデータとローダーの導入、コンテンツプラットフォーム、起動パイプライン）を実装しています。実機での起動確認、3 プラットフォームのパッケージ証跡、署名・公証はまだ完了していません。CI の成果物は**未署名のテストビルドであり、正式版ではありません**。

## 概要

Mnelys はゼロから独自開発しているクロスプラットフォーム対応 Minecraft: Java Edition ランチャーです。通常のプレイヤーと Mod 開発者が、同じアカウント、インスタンス、タスク、キャッシュ、プロセスモデルを利用できる製品を目指しています。

Prism Launcher のソースコード、アセット、Git 履歴、内部データモデルは継承しません。互換機能は公開プロトコル、公式 API、公開ファイル形式、および独自に作成したテストフィクスチャだけを基に実装します。

## 予定している機能

- Microsoft、オフライン、外部 Yggdrasil アカウント
- Vanilla、Fabric、Quilt、Forge、NeoForge などのインスタンス
- Java の自動検出、ダウンロード、バージョン選択
- Modrinth、CurseForge、ローカルコンテンツの導入
- ダウンロードキャッシュ、ハッシュ検証、ミラー切替、障害復旧
- Terracotta マルチプレイヤーコンポーネント連携
- Gradle ビルド、デプロイ、Debug Run、JDWP、Developer Daemon
- ローカルルールを優先し、ユーザーが明示的に許可した AI クラッシュ診断

上記はロードマップであり、現時点ですべて実装済みという意味ではありません。

## 現在の進捗

M1–M6 のコア機能は実装済みです。実機での証跡と配布物はまだ残っています。

- [x] 独立リポジトリ、ライセンス、ADR、セキュリティ基準
- [x] Tauri v2 のランタイム／UI 基準（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- [x] フロントエンドのフレームワークと技術スタック（[ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)）
- [x] デザインシステム、ライト／ダーク／システムテーマ、既定のピンクとカスタムアクセントカラー
- [x] 自前描画のタイトルバー（macOS はネイティブの信号機を維持）、単一のローカライズ済み製品名
- [x] シングルウィンドウシェル、レスポンシブナビゲーション、タスクセンター、ログパネル、設定、開発者モード
- [x] 10 言語のカタログ：簡体字中国語、繁体字中国語、英語、日本語、韓国語、フランス語、ドイツ語、スペイン語、ブラジルポルトガル語、ロシア語
- [x] インターフェースアイコンを Lucide（ISC）に統一（[ICONS.md](docs/ICONS.md)）
- [x] Rust Cargo workspace のレイヤー構成と v3 コマンド契約（39 コマンド）
- [x] Vanilla の導入と起動：メタデータ、ハッシュ検証、ミラー切り替え、Java の検出と導入、引数の展開、プロセスとログ
- [x] オフライン、Yggdrasil、Microsoft デバイスコードのアカウント。キーストア保存でトークンは IPC を越えない
- [x] Fabric / Quilt / Legacy Fabric のメタデータローダー
- [x] Forge / NeoForge インストーラーパイプライン
- [x] Modrinth / CurseForge の検索、導入、削除、インスタンスマニフェストの入出力
- [x] GitHub Actions：フロントエンド検査、3 プラットフォームの Rust マトリクス、デスクトップパッケージング
- [ ] 実機でメインメニューまでの起動、DPI、3 プラットフォームの視覚回帰
- [ ] 3 プラットフォームの成果物サイズ、署名、公証の証跡

旧 Avalonia 実装で得たパッケージングの証跡は [M0 パッケージング PoC レポート](docs/M0_PACKAGING_POC.md)にあります。これは履歴であり、**Tauri 基盤の証跡ではありません**。

## 対応プラットフォーム

| プラットフォーム            | 1.0 目標 | 配布物                                      |
| --------------------------- | -------: | ------------------------------------------- |
| Windows 10/11 x64           |   Tier A | NSIS インストーラーとポータブル実行ファイル |
| macOS 13+ Apple Silicon     |   Tier A | 署名・公証済み DMG                          |
| Linux x86_64                |   Tier A | AppImage（`.deb` も同梱）                   |
| macOS Intel/x64             |   非対応 | 配布物なし                                  |
| Windows ARM64 / Linux ARM64 |     延期 | 1.0 以降に検討                              |

## 技術構成

- **Tauri 2.x** をデスクトップシェル兼ランタイムとして採用（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- **Rust** でコアを実装：アカウント、インスタンス、Java 管理、ダウンロード、キャッシュ、起動パイプライン、IPC。コアは Cargo workspace（domain / application / infrastructure / presentation）で、`src-tauri` は組み立て役に徹します
- プラットフォーム WebView：Windows は WebView2、macOS は WKWebView、Linux は WebKitGTK
- フロントエンドは **React 19 + TypeScript**（[ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)）。Vite でビルドし、`react-router` のハッシュルーティング、サーバー状態に TanStack Query、一時状態に Zustand、UI テキストに i18next、インターフェースアイコンに Lucide（ISC、[ICONS.md](docs/ICONS.md)）を使用
- 言語カタログは `import.meta.glob` で自動検出します。言語の追加は JSON ファイル 1 つだけで、`src/i18n/catalogs.test.ts` が全カタログのキー集合を `en` と完全一致させます。アクセントカラーと命名規則は [ADR 0007](docs/adr/0007-accent-color-and-localization-policy.md) に記録しています
- Rust ツールチェーンは `rust-toolchain.toml`、Node.js と pnpm は `package.json` で固定
- ルートウィンドウは不透明を維持：Mica、Acrylic、macOS vibrancy、デスクトップサンプリングは禁止。グローバルスタイルの `backdrop-filter: none !important` が最終的な防波堤です
- フロントエンドは明示的に宣言した Tauri コマンドを通じてのみネイティブ機能にアクセスし、`invoke` は `src/ipc/` 以外に書けません。ケイパビリティはウィンドウ単位で宣言・レビューします

依存方向は `presentation → application → domain` に限定します。Infrastructure、Provider、Platform の各クレートが内側の層で定義した Port を実装し、Tauri の型はコマンドとアダプターの境界に留めます。詳細は[アーキテクチャ基準](docs/ARCHITECTURE.md)を参照してください。

## ローカル実行

必要な環境（2026-09-30 以降）：

- `rust-toolchain.toml` で固定された Rust stable ツールチェーン
- Node.js 24+ と pnpm 11.7.0（`packageManager` フィールドで固定。corepack での有効化を推奨）
- プラットフォーム別のビルド依存：Windows は WebView2 ランタイムと MSVC ビルドツール（Visual Studio Build Tools の「C++ によるデスクトップ開発」ワークロード。`link.exe` を提供）、macOS は Xcode command line tools、Linux は WebKitGTK と `libayatana-appindicator` などの Tauri システムパッケージ
- Git 2.40+

> Windows では MSYS2/MinGW の GNU ツールチェーンを MSVC の代わりに使えません。`x86_64-pc-windows-gnu` はコンパイル、`cargo clippy`、domain / application / infrastructure のテストを通過しますが、`mnelys-presentation` と `src-tauri` のテストバイナリは WinRT API set に依存するためローカルの GNU 環境では読み込めません。完全なテストは CI の MSVC / Linux / macOS 実行を基準とします。

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys

pnpm install                                       # フロントエンド依存
cargo fetch --manifest-path src-tauri/Cargo.toml   # Rust 依存

pnpm tauri:dev                                     # デスクトップアプリを起動（Vite dev server も同時に起動）
```

`package.json` のスクリプト：

| スクリプト                            | 用途                                    |
| ------------------------------------- | --------------------------------------- |
| `pnpm dev`                            | Vite のみ起動（ブラウザーで UI を確認） |
| `pnpm build`                          | 型チェックして `dist/` へビルド         |
| `pnpm typecheck`                      | `tsc --noEmit`                          |
| `pnpm lint`                           | ESLint                                  |
| `pnpm format:check`                   | Prettier チェック                       |
| `pnpm test`                           | Vitest + Testing Library                |
| `pnpm tauri:dev` / `pnpm tauri:build` | Tauri の開発とパッケージング            |

## CI とパッケージング

- [`CI`](.github/workflows/ci.yml) はフロントエンドのフォーマット、Lint、型チェック、テスト、ビルドに加え、Windows / macOS / Linux で `cargo fmt`、`cargo clippy -D warnings`、`cargo test` を実行します。
- [`Desktop packages`](.github/workflows/desktop-package.yml) は手動実行または `v*` タグで NSIS インストーラー、macOS arm64 DMG、AppImage を生成してアップロードします。成果物は未署名で 14 日間保持され、ビルド証跡としてのみ提供されます。署名と公証にはリポジトリ外の認証情報が必要です。
- **テスト成果物のダウンロード**：GitHub の Actions → `Desktop packages` → `Run workflow` を実行し、完了後にその実行の Artifacts から `mnelys-windows-x64-unsigned`、`mnelys-macos-arm64-unsigned`、または `mnelys-linux-x64-unsigned` を取得します。Windows の成果物には NSIS インストーラーとポータブル版 `mnelys.exe` が含まれます。
- 通常の検証は CI を基準とし、ローカルのデスクトップビルドは成果物や対話的デバッグが必要な例外時だけ行います。

> `src-tauri/` はコンパイル時にフロントエンド成果物の存在を検証します。`cargo check`、`cargo test`、`cargo clippy` を単体で実行する場合は、先に `pnpm build` を実行してください。`dist/` がないと `tauri::generate_context!` が失敗します。

## パッケージング

3 プラットフォームの配布物はカスタムスクリプトではなく Tauri bundler が生成します。

| プラットフォーム | bundle ターゲット | 配布物                                 |
| ---------------- | ----------------- | -------------------------------------- |
| Windows x64      | `nsis`            | インストーラーとポータブル実行ファイル |
| macOS arm64      | `dmg`             | 署名・公証済み DMG                     |
| Linux x86_64     | `appimage`、`deb` | AppImage（主）と `.deb`                |

```bash
pnpm tauri:build -- --bundles nsis
pnpm tauri:build -- --bundles dmg
pnpm tauri:build -- --bundles appimage,deb
```

Windows は `x86_64-pc-windows-msvc`、macOS は `aarch64-apple-darwin` を明示的に指定します。

対象 OS のツール、署名要件、現在の制限は[パッケージングガイド](packaging/README.md)を参照してください。

## リポジトリ構成

```text
Mnelys/
├─ assets/                 ブランド原本
├─ crates/                 Rust コアのレイヤー：domain / application / infrastructure / presentation
├─ docs/                   計画、設計、セキュリティ、対応表、ADR
├─ packaging/              3 プラットフォームのパッケージング手順（成果物は Tauri bundler が生成）
├─ src/                    React 19 フロントエンド
│  ├─ app/                 アプリ構成：providers、router
│  ├─ components/          再利用コンポーネント
│  ├─ ipc/                 唯一の Tauri コマンド／エラー境界
│  ├─ i18n/                10 言語のカタログ（自動検出）
│  ├─ routes/              ページコンポーネント
│  ├─ state/               ページをまたぐ一時状態
│  ├─ styles/              トークンとグローバルスタイル
│  └─ test/                テスト初期化
├─ src-tauri/              Tauri v2 シェル、ウィンドウ設定、組み立て役
│  ├─ capabilities/        ウィンドウ単位のケイパビリティ
│  ├─ icons/               アプリアイコン
│  ├─ src/                 組み立て：状態の設置、コマンドの登録、ランタイム起動
│  └─ tests/               フロントエンドとのコマンド契約テスト
├─ Cargo.toml / Cargo.lock  ワークスペース定義と依存ロック
├─ package.json / pnpm-lock.yaml
├─ rust-toolchain.toml
├─ tsconfig.json / vite.config.ts / eslint.config.js
└─ .prettierrc.json
```

Rust コアは Cargo workspace です：`mnelys-domain`（失敗の語彙）、`mnelys-application`（ユースケースとポート）、`mnelys-infrastructure`（アダプター）、`mnelys-presentation`（Tauri コマンドとハンドラー）。`src-tauri` はそれらを組み立てるだけです。Provider と platform のクレートは、必要とする機能が現れた段階で分割します。

## コントリビューションと独立実装

変更を提出する前に [CONTRIBUTING.md](CONTRIBUTING.md)を確認してください。Prism Launcher、MultiMC、HMCL、PCL2、その他のランチャーからソースコード、アセット、翻訳、認証情報、CI、パッケージスクリプト、内部モデルをコピーまたは改変して持ち込むことは禁止します。

脆弱性の報告は [SECURITY.md](docs/SECURITY.md)に従ってください。公開 Issue に認証情報や未加工の個人データを投稿しないでください。

## ライセンスと免責事項

ソースコードは [MIT License](LICENSE)で公開します。

Mnelys は Mojang Studios または Microsoft の公式製品ではなく、両社から承認を受けたものでも、両社と提携しているものでもありません。Minecraft は各権利者の商標です。ブランド画像の原作者情報と公開再配布条件は、最初の公開リリース前に記録する必要があります。
