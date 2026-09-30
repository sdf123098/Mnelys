<p align="center">
  <img src="assets/branding/mnelys-icon-source.jpg" width="220" alt="Mnelys アイコン">
</p>

<h1 align="center">Mnelys · 忆涟</h1>

<p align="center">
  プレイヤーと Mod 開発者のための AI ネイティブ Minecraft: Java Edition ランチャー／開発ワークスペース
</p>

<p align="center">
  <a href="README.md">简体中文</a> · <a href="README.en.md">English</a> · 日本語
</p>

> [!IMPORTANT]
> Mnelys は技術基盤を **Tauri v2 + React 19** へ切り替えました（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md) と [ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md) を参照）。現在の `main` には Rust コアの骨組みと React フロントエンドが入っており、**ローカルで起動できます**。ただし Minecraft のインストール／起動機能と 3 プラットフォームの配布物はまだありません。

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

現在は M0「独立プロジェクトの確立と技術検証」で、技術スタックの変更により再スタートしています。

- [x] 独立リポジトリ、ライセンス、ADR、セキュリティ基準
- [x] Tauri v2 のランタイム／UI 基準を確立（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- [x] .NET／Avalonia の実装を削除（実装自体は本リポジトリの Git 履歴に残っています）
- [x] フロントエンドのフレームワークと技術スタックを決定（[ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)）
- [x] Rust コアの骨組み、コマンド境界、フロントエンドとの契約テスト
- [x] React フロントエンド、3 言語 i18n、コンポーネントテスト
- [ ] Tauri bundler による Windows／macOS／Linux のパッケージング PoC
- [ ] 最初の Vanilla 垂直スライス

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
- **Rust** でコアを実装：アカウント、インスタンス、Java 管理、ダウンロード、キャッシュ、起動パイプライン、IPC
- プラットフォーム WebView：Windows は WebView2、macOS は WKWebView、Linux は WebKitGTK
- フロントエンドは **React 19 + TypeScript**（[ADR 0006](docs/adr/0006-frontend-framework-and-application-stack.md)）。Vite でビルドし、`react-router` のハッシュルーティング、サーバー状態に TanStack Query、一時状態に Zustand、UI テキストに i18next を使用
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

> Windows では MSYS2/MinGW の GNU ツールチェーンを MSVC の代わりに使えません。`x86_64-pc-windows-gnu` はコンパイルと `cargo clippy` を通過しますが、テストバイナリは読み込み時に `0xC0000139`（`STATUS_ENTRYPOINT_NOT_FOUND`）で失敗します。Rust の GNU ターゲットは msvcrt ベース、MSYS2 の UCRT64 ツールチェーンは UCRT ベースだからです。

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
├─ docs/                   計画、設計、セキュリティ、対応表、ADR
├─ packaging/              3 プラットフォームのパッケージング手順（成果物は Tauri bundler が生成）
├─ src/                    React 19 フロントエンド
│  ├─ app/                 アプリ構成：providers、router
│  ├─ components/          再利用コンポーネント
│  ├─ ipc/                 唯一の Tauri コマンド／エラー境界
│  ├─ i18n/                zh-Hans / en / ja カタログ
│  ├─ routes/              ページコンポーネント
│  ├─ state/               ページをまたぐ一時状態
│  ├─ styles/              トークンとグローバルスタイル
│  └─ test/                テスト初期化
├─ src-tauri/              Tauri v2 シェルと Rust コア
│  ├─ capabilities/        ウィンドウ単位のケイパビリティ
│  ├─ icons/               アプリアイコン
│  ├─ src/                 commands、error、`run()` エントリー
│  └─ tests/               フロントエンドとのコマンド契約テスト
├─ package.json / pnpm-lock.yaml
├─ rust-toolchain.toml
├─ tsconfig.json / vite.config.ts / eslint.config.js
└─ .prettierrc.json
```

`crates/` の分割（domain / application / presentation / infrastructure / providers / platform）は M1 で導入予定です。M0 の Rust コードは `src-tauri/src/` に置きます。

## コントリビューションと独立実装

変更を提出する前に [CONTRIBUTING.md](CONTRIBUTING.md)を確認してください。Prism Launcher、MultiMC、HMCL、PCL2、その他のランチャーからソースコード、アセット、翻訳、認証情報、CI、パッケージスクリプト、内部モデルをコピーまたは改変して持ち込むことは禁止します。

脆弱性の報告は [SECURITY.md](docs/SECURITY.md)に従ってください。公開 Issue に認証情報や未加工の個人データを投稿しないでください。

## ライセンスと免責事項

ソースコードは [MIT License](LICENSE)で公開します。

Mnelys は Mojang Studios または Microsoft の公式製品ではなく、両社から承認を受けたものでも、両社と提携しているものでもありません。Minecraft は各権利者の商標です。ブランド画像の原作者情報と公開再配布条件は、最初の公開リリース前に記録する必要があります。
