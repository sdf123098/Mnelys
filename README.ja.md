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
> Mnelys は技術基盤を .NET／Avalonia から **Tauri v2** へ移行中です（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md) を参照）。現在の `main` にはプロジェクト文書、ブランド素材、意思決定の記録しかなく、**実行可能な実装はまだありません**。Minecraft のインストール／起動機能も未実装です。

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
- [ ] フロントエンドフレームワークの決定（ADR 0006、保留中）
- [ ] Rust コアの骨組みと Tauri コマンド境界
- [ ] Tauri bundler による Windows／macOS／Linux のパッケージング PoC
- [ ] 最初の Vanilla 垂直スライス

旧 Avalonia 実装で得たパッケージングの証跡は [M0 パッケージング PoC レポート](docs/M0_PACKAGING_POC.md)にあります。これは履歴であり、**Tauri 基盤の証跡ではありません**。

## 対応プラットフォーム

| プラットフォーム | 1.0 目標 | 配布物 |
|---|---:|---|
| Windows 10/11 x64 | Tier A | NSIS インストーラーとポータブル実行ファイル |
| macOS 13+ Apple Silicon | Tier A | 署名・公証済み DMG |
| Linux x86_64 | Tier A | AppImage（`.deb` も同梱） |
| macOS Intel/x64 | 非対応 | 配布物なし |
| Windows ARM64 / Linux ARM64 | 延期 | 1.0 以降に検討 |

## 技術構成

- **Tauri 2.x** をデスクトップシェル兼ランタイムとして採用（[ADR 0005](docs/adr/0005-runtime-and-ui-baseline-tauri.md)）
- **Rust** でコアを実装：アカウント、インスタンス、Java 管理、ダウンロード、キャッシュ、起動パイプライン、IPC
- プラットフォーム WebView：Windows は WebView2、macOS は WKWebView、Linux は WebKitGTK
- フロントエンドフレームワークは**未決定**（ADR 0006）。決定まで本番 UI コードはコミットしません
- Rust ツールチェーンは `rust-toolchain.toml` で固定し、Node.js とパッケージマネージャーもリポジトリ内で固定します
- ルートウィンドウは不透明を維持：Mica、Acrylic、macOS vibrancy、デスクトップサンプリングは禁止
- フロントエンドは明示的に宣言した Tauri コマンドを通じてのみネイティブ機能にアクセスし、ケイパビリティはウィンドウ単位で宣言・レビューします

依存方向は `presentation → application → domain` に限定します。Infrastructure、Provider、Platform の各クレートが内側の層で定義した Port を実装し、Tauri の型はコマンドとアダプターの境界に留めます。詳細は[アーキテクチャ基準](docs/ARCHITECTURE.md)を参照してください。

## ローカル実行

必要な環境（2026-09-30 以降）：

- `rust-toolchain.toml` で固定された Rust stable ツールチェーン
- リポジトリ内で固定された Node.js とパッケージマネージャー
- プラットフォーム別のビルド依存：Windows は WebView2 と MSVC ビルドツール、macOS は Xcode command line tools、Linux は WebKitGTK と `libayatana-appindicator` などの Tauri システムパッケージ
- Git 2.40+

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys
```

現在の `main` には実行可能な Tauri プロジェクトはまだありません。スキャフォールドは ADR 0006 でフロントエンドフレームワークを確定した後に追加します。その時点でのローカル起動とビルドは `npm run tauri dev` と `npm run tauri build` で、正確なスクリプト名はフロントエンドと同時に追加される `package.json` に従います。

## パッケージング

3 プラットフォームの配布物はカスタムスクリプトではなく Tauri bundler が生成します。

| プラットフォーム | bundle ターゲット | 配布物 |
|---|---|---|
| Windows x64 | `nsis` | インストーラーとポータブル実行ファイル |
| macOS arm64 | `dmg` | 署名・公証済み DMG |
| Linux x86_64 | `appimage`、`deb` | AppImage（主）と `.deb` |

```bash
npm run tauri build -- --bundles nsis
npm run tauri build -- --bundles dmg
npm run tauri build -- --bundles appimage,deb
```

対象 OS のツール、署名要件、現在の制限は[パッケージングガイド](packaging/README.md)を参照してください。

## リポジトリ構成

```text
Mnelys/
├─ assets/                 ブランド原本
├─ docs/                   計画、設計、セキュリティ、対応表、ADR
├─ packaging/              3 プラットフォームのパッケージング手順（成果物は Tauri bundler が生成）
└─ （未追加）              `src-tauri/` とフロントエンド。ADR 0006 の後に作成
```

## コントリビューションと独立実装

変更を提出する前に [CONTRIBUTING.md](CONTRIBUTING.md)を確認してください。Prism Launcher、MultiMC、HMCL、PCL2、その他のランチャーからソースコード、アセット、翻訳、認証情報、CI、パッケージスクリプト、内部モデルをコピーまたは改変して持ち込むことは禁止します。

脆弱性の報告は [SECURITY.md](docs/SECURITY.md)に従ってください。公開 Issue に認証情報や未加工の個人データを投稿しないでください。

## ライセンスと免責事項

ソースコードは [MIT License](LICENSE)で公開します。

Mnelys は Mojang Studios または Microsoft の公式製品ではなく、両社から承認を受けたものでも、両社と提携しているものでもありません。Minecraft は各権利者の商標です。ブランド画像の原作者情報と公開再配布条件は、最初の公開リリース前に記録する必要があります。
