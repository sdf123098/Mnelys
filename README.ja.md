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
> Mnelys は現在 M0 技術検証段階です。このリポジトリには実行可能な Avalonia シェルと各プラットフォーム向けパッケージング PoC が含まれていますが、Minecraft のインストール／起動機能はまだ完成していません。

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

現在は M0「独立プロジェクトの確立と技術検証」です。

- [x] 独立リポジトリ、ライセンス、ADR、セキュリティ基準
- [x] .NET 10、C# 14、Avalonia 12 のバージョン固定
- [x] Mica／Acrylic を使わない不透明な Avalonia Desktop シェル
- [x] Windows x64 自己完結型シングル EXE PoC
- [x] macOS arm64 自己完結型 payload と `.app` 構造 PoC
- [x] Linux x64 自己完結型 payload と AppDir 構造 PoC
- [ ] macOS 実機での署名、公証、DMG 検証
- [ ] Linux AppImage ビルダーとディストリビューション検証
- [ ] 最初の Vanilla 垂直スライス

検証結果と残っている作業は [M0 パッケージング PoC レポート](docs/M0_PACKAGING_POC.md)を参照してください。

## 対応プラットフォーム

| プラットフォーム | 1.0 目標 | 配布物 |
|---|---:|---|
| Windows 10/11 x64 | Tier A | 自己完結型シングル `Mnelys.exe` |
| macOS 13+ Apple Silicon | Tier A | 署名・公証済み DMG |
| Linux x86_64 | Tier A | AppImage |
| macOS Intel/x64 | 非対応 | 配布物なし |
| Windows ARM64 / Linux ARM64 | 延期 | 1.0 以降に検討 |

## 技術構成

- C# 14 / .NET 10 LTS
- Avalonia 12、AXAML、Compiled Bindings
- MVVM と単方向状態更新
- Central Package Management
- 今後のドメイン実装では `System.Text.Json` Source Generation
- 今後のコア基盤では SQLite、`HttpClientFactory`、構造化ログ

依存方向は Presentation → Application → Domain に限定します。Infrastructure、Provider、Platform Adapter は内側の層で定義した Port を実装します。詳細は[アーキテクチャ基準](docs/ARCHITECTURE.md)を参照してください。

## ローカル実行

必要な環境：

- .NET SDK `10.0.302`、または `global.json` が許可する新しい `10.0.3xx` パッチ
- Git 2.40+

```powershell
git clone https://github.com/sdf123098/Mnelys.git
cd Mnelys
dotnet restore Mnelys.slnx
dotnet run --project src/Mnelys.Desktop/Mnelys.Desktop.csproj
```

Release ビルドとフォーマットの確認：

```powershell
dotnet build Mnelys.slnx -c Release
dotnet format Mnelys.slnx --verify-no-changes --no-restore
```

## パッケージング PoC

Windows x64：

```powershell
.\packaging\windows\Publish-SingleFile.ps1
```

macOS arm64：

```bash
./packaging/macos/build-dmg.sh
```

Linux x64：

```bash
./packaging/linux/build-appimage.sh
```

対象 OS のツール、署名要件、現在の制限は[パッケージングガイド](packaging/README.md)を参照してください。

## リポジトリ構成

```text
Mnelys/
├─ assets/                 ブランド原本
├─ docs/                   計画、設計、セキュリティ、対応表、ADR
├─ packaging/              Windows、macOS、Linux のパッケージスクリプト
├─ src/Mnelys.Desktop/     現在の Avalonia Desktop シェル
├─ Directory.Build.props   リポジトリ全体のビルドルール
├─ Directory.Packages.props 依存パッケージの中央管理
├─ global.json             .NET SDK の固定
└─ Mnelys.slnx             ソリューション
```

## コントリビューションと独立実装

変更を提出する前に [CONTRIBUTING.md](CONTRIBUTING.md)を確認してください。Prism Launcher、MultiMC、HMCL、PCL2、その他のランチャーからソースコード、アセット、翻訳、認証情報、CI、パッケージスクリプト、内部モデルをコピーまたは改変して持ち込むことは禁止します。

脆弱性の報告は [SECURITY.md](docs/SECURITY.md)に従ってください。公開 Issue に認証情報や未加工の個人データを投稿しないでください。

## ライセンスと免責事項

ソースコードは [MIT License](LICENSE)で公開します。

Mnelys は Mojang Studios または Microsoft の公式製品ではなく、両社から承認を受けたものでも、両社と提携しているものでもありません。Minecraft は各権利者の商標です。ブランド画像の原作者情報と公開再配布条件は、最初の公開リリース前に記録する必要があります。
