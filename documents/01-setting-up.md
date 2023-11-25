# 準備

## イントロダクション

ソフトウェアを作成するとき、[Architecture Decision Record](https://adr.github.io/)と呼ばれるテクニックを使用して、アーキテクチャを文書かすることをお勧めします。

ADRは、タイトル、状態、文脈、決定そして特別な設計の選択の結果を記録するマークダウン文書であるだけではありません。

決定されたとき、その決定が現実世界でどのように展開されるかを説明する小さな概念実証を作成することが役に立つことがあります。

### このガイド

いくつかのADRを作成したいくつかのプロジェクトを通じて実行した後、私はそれらが再利用可能であることを認識しました。
アーキテクチャの決定を証明することが要求される概念実証では、ほぼチュートリアルのようになります。

このガイドは、プロダクション用のRustのWebアプリケーションを構築する方法を紹介します。

次のアプリケーションは、ここで文書化した決定を使用して構築されています。

### ショウケース

次のプロジェクトは、これらのガイドラインを使用して構築されました。

* [Bionic GPT](https://github.com/purton-tech/bionicgpt?campaign=rustonnails)
* [SkyTrace](https://github.com/purton-tech/skytrace?campaign=rustonnails)

### アーキテクチャ

![アーキテクチャ](https://rust-on-nails.com/architecture-diagram.svg)

## コードとしての開発環境

[Visual Studio Code Remote - Containers](https://code.visualstudio.com/docs/remote/containers)拡張昨日は、完全な機能を持った開発環境としてDockerコンテナを使用することを可能sにします。
これは、次の問題を修正します。

* 自分以外の開発者がすぐに速度を上げることを可能にする
* 「自分のマシンでは動作する」のような問題を回避する
* 開発環境をgitで管理することを可能にする。

### インストール

VSCodeにdevcontainer拡張機能をインストールして、そのあとでRust環境を準備します。

![Dev Containers](https://rust-on-nails.com/containers-extension.png)

### Rust on Nailsのインストール

私たちには、フルスタックなRustアプリケーションを作成するために必要なすべてのツールを持った、事前に構成した開発環境があります。

開始するためにあなたのプロジェクト用のフォルダを作成します。
どのフォルダにディレクトリを変更して、次を実行します。

```sh
mkdir project-name
cd project-name
```

#### MacOSとLinux

```sh
curl -L https://github.com/purton-tech/rust-on-nails/archive/main.tar.gz | \
  tar xvz --strip=2 rust-on-nails-main/nails-devcontainer/ \
  && rm devcontainer-template.json
```

#### Windows

```powershell
curl -L https://github.com/purton-tech/rust-on-nails/archive/main.tar.gz | \
  tar xvz --strip=2 rust-on-nails-main/nails-devcontainer/ \
  && del devcontainer-template.json
```

> `.devcontainer/docker-compose.yml`を次の通り修正する必要があります。

```yaml
 version: '3.4'
 services:

   db:
     image: postgres:14-alpine
     environment:
       POSTGRES_PASSWORD: testpassword
       POSTGRES_USER: postgres
     healthcheck:
-      test: ["CMD-SHELL", "pg_isready -U postgres"]
+      test: ["CMD-SHELL", "pg_isready -U vscode"]
       interval: 10s
       timeout: 5s
       retries: 5
```

#### VS Code

Visual Studio Code内にそのフォルダを読み込みます。
あなたは、VS Codeの左下隅に、緑のアイコンを確認できるはずです。
これをクリックして、`Open in Container`を選択します。

コンテナがダウンロードされると、あなたは次のフォルダ構成を持った事前に構成された開発環境を保ちます。

フォルダ構成がどのようになっているか確認します。

```text
.
└── .devcontainer/
    ├── .bash_aliases
    ├── .githooks/
    │   └── precommit
    ├── devcontainer.json
    ├── docker-compose.yml
    └── Dockerfile
└── .gitignore
└── README.md
```

### Gitの準備

VSCodeのターミナルを開いて、実行します。

<!-- markdownlint-disable MD014 -->
```sh
$ cargo new --vcs=none crates/axum-server
     Created binary (application) `crates/axum-server` package
```
<!-- markdownlint-enable -->

### ワークスペースの追加

私たちのWebアプリケーション用のワークスペースを作成します。
ルートフォルダに新しい`Cargo.toml`ファイルを作成して、次を追加します。

```toml
[workspace]
members = [
    "crates/*",
]
```

再度、VS Codeのターミナルを開いて、次を実行します。

<!-- markdownlint-disable MD014 -->
```sh
$ cargo new --vcs=none crates/axum-server
     Created binary (application) `crates/axum-server` package
```
<!-- markdownlint-enable -->

あなたは、次のようなフォルダ構造を持つはずです。

```text
├── .devcontainer/
│   └── ...
└── crates/
│         axum-server/
│         │  └── main.rs
│         └── Cargo.toml
└── Cargo.toml
```

### テスト

あなたの開発環境をテストします。

<!-- markdownlint-disable MD014 -->
```sh
$ cargo run
   Compiling app v0.1.0 (/workspace/app)
    Finished dev [unoptimized + debuginfo] target(s) in 1.16s
     Running `target/debug/app`
Hello, world!
```
<!-- markdownlint-enable -->

### あなたのコードをコミット

`/workspace`フォルダから、次を実行します。

<!-- markdownlint-disable MD014 -->
```sh
$ git add .
$ git commit -m "Initial Commit"
```
<!-- markdownlint-enable -->
