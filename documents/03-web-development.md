# Web開発

## Webサーバーとルーティング

私たちは、[Actix Web](https://actix.rs/)、[Tokio Axum](https://github.com/tokio-rs/axum)そして[Rocket](https://rocket.rs/)を確認しました。
Axumは、とても積極的にメンテナンスされており、増分ビルド時間が最も早いため選択されました。

ほとんどのRustのWebサーバープロジェクトは、同様に操作します。
それは、ルートとそのルートに応答する関数を構成することです。

ルートに応答する関数は、パラメーターを持てます。
`struct`、データベースプールまたはデータかもしれないこれらのパラメーターは、フレームワークによって関数に渡されます。

### 構成を扱う

私たちは、設定自身のファイルに設定を分離します。
`crates/axum-server/src/config.rs`を作成してください。

```rust
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn new() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

        Config {
            database_url,
        }
    }
}
```

### エラーを扱う

現在、常に`unwrap`しないようにするために、私たちはエラーをどのように扱うか考える良い時間です。

`crates/axum-server/src/errors.rs`と呼ばれるファイルを作成して、次のコードを追加します。

```rust
use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use db::{PoolError, TokioPostgresError};

#[derive(Debug)]
pub enum CustomError {
    FaultySetup(String),
    Database(String),
}

/// "{}"書式記述の使用を許可
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::FaultySetup(ref cause) => write!(f, "Setup Error: {}", cause),
            CustomError::Database(ref cause) => write!(f, "Database Error: {}", cause),
        }
    }
}

/// 次のエラーはブラウザに表示されますか?
impl IntoResponse for CustomError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            CustomError::Database(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            CustomError::FaultySetup(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        };

        format!("status={}, message={}", status, error_message).into_response()
    }
}

impl From<axum::http::uri::InvalidUri> for CustomError {
    fn from(err: axum::http::uri::InvalidUri) -> Self {
        CustomError::FaultySetup(err.to_string())
    }
}

impl From<TokioPostgresError> for CustomError {
    fn from(err: TokioPostgresError) -> Self {
        CustomError::Database(err.to_string())
    }
}

impl From<PoolError> for CustomError {
    fn from(err: PoolError) -> Self {
        CustomError::Database(err.to_string())
    }
}
```

### Axumのインストール

`crates/axum-server`フォルダ内にいることを確認して、次のコマンドを使用して`Cargo.toml`にAxumを追加します。

```sh
cargo add axum
cargo add tokio --no-default-features -F tokio/macros,tokio/fs,tokio/rt-multi-thread
cargo add --path ../db      # ワークスペースの`db`クレートを依存関係に追加
```

そして、次で`crates/axum-server/src/main.rs`を置き換えます。

```rust
use std::net::SocketAddr;

use axum::extract::Extension;
use axum::response::Json;
use axum::routing::get;
use axum::Router;

mod config;
mod errors;

use crate::errors::CustomError;
use db::User;

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // ルートでアプリケーションを構築
    let app = Router::new()
        .route("/", get(users))
        .layer(Extension(config))
        .layer(Extension(pool.clone()));

    // アプリケーションを起動
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn users(Extension(pool): Extension<db::Pool>) -> Result<Json<Vec<User>>, CustomError> {
    let client = pool.get().await?;
    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Json(users))
}
```

### サーバーの監視

私たちは、サーバーを`cargo run`で起動できますが、Rust on Nailsには、コードの変更を監視してサーバーを再起動する組み込みのエイリアスが付属しています。

また、それは、増分ビルド時間を早くするために[Mold](https://github.com/rui314/mold)と呼ばれるとても早いリンカーを使用しています。

`app(axum-server)`フォルダ内で次のコマンドを発行してください。

```sh
cw
```

そしてブラウザで`http://localhost:3000`を指定することができ、Webサーバーはユーザーの簡素なテキストを配信します。

## サーバー側のコンポーネント

[Dioxus](https://dioxuslabs.com/)フレームワークは、サーバー側でレンダリングされるコンポーネントからユーザーインターフェースを構築する能力を与えます。
それは、それらの[コンポーネントドキュメント](https://dioxuslabs.com/guide/components/index.html)で確認する価値があります。

> Dioxus（だいおくさす）は、仮想DOMを使用してWebのフロントエンド、デスクトップアプリ、モバイルアプリまたはテキストのUIを作成するクレートです。

### ui-componentsクレートの作成

> `/workspace`で次を実行します。

```sh
cargo init --lib crates/ui-components
```

### Dioxusのインストール

```sh
cd crates/ui-components
cargo add dioxus@0.2 --features ssr
```

### レイアウトコンポーネントの作成

レイアウトは、HTMLページのまわりを定義します。
それは、最終的な出力の一般的な外観と雰囲気を定義する場所です。

`crates/ui-components/src/layout.rs`と呼ぶファイルを作成してください。

```rust
#![allow(non_snake_case)]

use dioxus::prelude::*;

/// 覚えておいてください: 所有されるプロップスは、PartialEqを実装しなければなりません!
#[derive(Props)]
pub struct AppLayoutProps<'a> {
    title: &'a str,
    children: Element<'a>,
}

pub fn Layout<'a>(cx: Scope<'a, AppLayoutProps<'a>>) -> Element {
    cx.render(rsx!(
        {
            LazyNodes::new(|f| f.text(format_args!("<!DOCTYPE html><html lang='en'>")))
        }
        head {
            title {
                "{cx.props.title}"
            }
            meta {
                charset: "utf-8"
            }
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1"
            }
        }
        body {
            &cx.props.children
        }
    ))
}
```

ユーザーのテーブルを表示するとても単純なユーザー画面を作成するために、このレイアウトを使用します。

`crates/ui-components`フォルダ内にいることを確認して、次のコマンドを使用して`Cargo.toml`に`db`クレートを追加します。

```sh
cargo add --path ../db
```

`crates/ui-components/src/users.rs`ファイルを作成します。

```rust
use dioxus::prelude::*;

use crate::layout::Layout;
use db::User;

struct Props {
    users: Vec<User>,
}

/// Vec<User>を受け取り、HTMLテーブルを作成
pub fn users(users: Vec<User>) -> String {
    // rsx!コンポーネントを作成する内部関数
    fn app(cx: Scope<Props>) -> Element {
        cx.render(rsx! {
            Layout {        // 私たちのLayoutを使用
                title: "Users Table",
                table {
                    thead {
                        tr {
                            th { "ID" }
                            th { "Email" }
                        }
                    }
                    tbody {
                        cx.props.users.iter().map(|user| rsx!(
                            tr {
                                td {
                                    strong {
                                        "{user.id}"
                                    }
                                }
                                td {
                                    "{user.email}"
                                }
                            }
                        ))
                    }
                }
            }
        })
    }

    // 私たちのコンポーネントを作成して、それを文字列にレンダリング
    let mut app = VirtualDom::new_with_props(app, Props { users });
    let _ = app.rebuild();

    dioxus::ssr::render_vdom(&app)
}
```

もし、私たちが、次のように見えるように`crates/ui-components/src/lib.rs`を更新した場合・・・

```rust
mod layout;
pub mod users;
```

最終的に、私たちはJSONよりもHTMLを生成するために`axum-server`コードを変更できます。

`axum-server`フォルダ内にいることを確認して、次のコマンドを使用して`Cargo.toml`に`ui-components`クレートを追加します。

```sh
cargo add --path ../ui-components
```

`crates/axum-server/src/main.rs`を更新します。

```rust
use std::net::SocketAddr;

use axum::extract::Extension;
use axum::response::Html;
use axum::routing::get;
use axum::Router;

mod config;
mod errors;

use crate::errors::CustomError;

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // ルートでアプリケーションを構築
    let app = Router::new()
        .route("/", get(users))
        .layer(Extension(config))
        .layer(Extension(pool.clone()));

    // アプリケーションを起動
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn users(Extension(pool): Extension<db::Pool>) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;
    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Html(ui_components::users::users(users)))
}
```

あなたは、下のスクリーンショットのような結果を得るはずです。

![Users table in HTML](https://rust-on-nails.com/layout-screenshot.png)

## フォーム

ブラウザは、クライアント側で[ビルトインされたフォーム検証](https://developer.mozilla.org/en-US/docs/Learn/Forms/Form_validation)をサポートしています。
私たちは、これをユーザーに良い体験を与えて、バックエンドのセキュリティを保証するためにサーバー側の検証と一緒に使用することができます。

### ブラウザ検証

次のフォームにおいて、私たちはEメール型と必須属性を使用しています。
ブラウザは、そのフィールドが妥当なEメールアドレス（とパスワード）で満たされるまで、フォームの提出をブロックします。

```html
<form>
  <label for="user_email">Email:</label>
  <input id="user_email" name="email" type="email" required />
  <button>Submit</button>
</form>
```

私たちは、Dioxusを使用してこれと同じフォームを記述できます。
ユーザーを追加するフォームで`crates/ui-components/src/users.rs`を更新します。

```rust
use dioxus::prelude::*;

use crate::layout::Layout;
use db::User;

struct Props {
    users: Vec<User>,
}

/// Vec<User>を受け取り、HTMLテーブルを作成
pub fn users(users: Vec<User>) -> String {
    // rsx!コンポーネントを作成する内部関数
    fn app(cx: Scope<Props>) -> Element {
        cx.render(rsx! {
            Layout {        // 私たちのLayoutを使用
                title: "Users Table",
                table {
                    thead {
                        tr {
                            th { "ID" }
                            th { "Email" }
                        }
                    }
                    tbody {
                        cx.props.users.iter().map(|user| rsx!(
                            tr {
                                td {
                                    strong {
                                        "{user.id}"
                                    }
                                }
                                td {
                                    "{user.email}"
                                }
                            }
                        ))
                    }
                }

                // これが新しいフォーム
                form {
                    action: "sign_up",
                    method: "POST",
                    label { r#for: "user_email", "Email:" }
                    input { id: "user_email", name: "email", r#type: "email", required: "true" }
                    button { "Submit" }
                }
            }
        })
    }

    // 私たちのコンポーネントを作成して、それを文字列にレンダリング
    let mut app = VirtualDom::new_with_props(app, Props { users });
    let _ = app.rebuild();

    dioxus::ssr::render_vdom(&app)
}
```

注意: `for`と`type`は、Rustのキーワードです。
私たちは、「for」と「type」の文字列リテラルそのままが必要であることをRustに認識させるために、それらに`r#`をプレフィックスする必要があります。

### フォーム提出を扱う

私たちは、HTTPボディをRustの構造体に変換するために[serde](https://serde.rs/)をインストールする必要があります。

```sh
cd crates/axum-server
cargo add serde@1.0 --features derive
```

[Axum](https://github.com/tokio-rs/axum)は、[Handlers](https://docs.rs/axum/latest/axum/handler/index.html)をサポートしている。
私たちは、多くのいろいろな方法で、1つの方法はフォームを実装するためにそれらを使用できます。
私たちは、データベースに新しいユーザーを保存するために`create_form`ハンドラを作成します。
`crates/axum-server/src/main.rs`を更新します。

```rust
use std::net::SocketAddr;

// axumのインポートを更新
use axum::extract::Extension;
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
// 新しいインポート
use serde::Deserialize;

mod config;
mod errors;

use crate::errors::CustomError;

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // ルートでアプリケーションを構築
    let app = Router::new()
        .route("/", get(users))
        .route("/sign_up", post(accept_form)) // 新しいルートを追加
        .layer(Extension(config))
        .layer(Extension(pool.clone()));

    // アプリケーションを起動
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn users(Extension(pool): Extension<db::Pool>) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;
    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Html(ui_components::users::users(users)))
}

/// サインアップ
#[derive(Deserialize)]
struct SignUp {
    email: String,
}

/// フォーム提出ハンドラ
async fn accept_form(
    Extension(pool): Extension<db::Pool>,
    Form(form): Form<SignUp>,
) -> Result<Redirect, CustomError> {
    let client = pool.get().await?;

    let email = form.email;
    // TODO: パスワードを受け取り、それをハッシュ化
    let hashed_password = String::from("aaaa");
    let _ = db::queries::users::create_user()
        .bind(&client, &email.as_str(), &hashed_password.as_str())
        .await?;

    // 303 ユーザーリストにリダイレクト
    Ok(Redirect::to("/"))
}
```

私たちは、`accept_form`ハンドラで`db::queries::users::create_user()`を使用しています。
私たちは、実際のSQLクエリを含めるために`crates/db/queries/users.sql`も更新する必要があります。

```sql
--: User()

--! get_users : User
SELECT
    id,
    email
FROM users;

-- `create_user`クエリを追加
--! create_user
INSERT INTO users (email, hashed_password)
VALUES (:email, :hased_password);
```

あなたは、次のようなスナップショットのような結果を確認するはずです。

![ユーザーフォーム](https://rust-on-nails.com/form-screenshot.png)

もし、あなたがフォームにEメールアドレスを追加して、submitを押した場合、、サーバーはそのリクエストを処理して、usersテーブルを更新します。

### サーバー側の検証

私たちのWebフォームは、ユーザー入力がEメールアドレスであることを検証します。
私たちは、サーバーでもユーザー入力がEメールアドレスであることを検証するべきです。
私たちは、`SignUp`構造体の検証を追加する[Validator](https://github.com/Keats/validator)を使用できます。

`Validator`クレートをインストールします。

```sh
cd crates/axum-server
cargo add validator@0.15 --features derive
```

`crates/axm-server/src/main.rs`を更新します。

```rust
use std::net::SocketAddr;

// axumのインポートを更新
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
// 新しいインポート
use serde::Deserialize;
use validator::Validate;

mod config;
mod errors;

use crate::errors::CustomError;

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // ルートでアプリケーションを構築
    let app = Router::new()
        .route("/", get(users))
        .route("/sign_up", post(accept_form)) // 新しいルートを追加
        .layer(Extension(config))
        .layer(Extension(pool.clone()));

    // アプリケーションを起動
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn users(Extension(pool): Extension<db::Pool>) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;
    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Html(ui_components::users::users(users)))
}

/// サインアップ
#[derive(Deserialize, Validate)]
struct SignUp {
    #[validate(email)]
    email: String,
}

/// フォーム提出ハンドラ
async fn accept_form(
    Extension(pool): Extension<db::Pool>,
    Form(form): Form<SignUp>,
) -> Result<Response, CustomError> {
    // エラーハンドラを追加
    if form.validate().is_err() {
        return Ok((StatusCode::BAD_REQUEST, "Bad request").into_response());
    }

    let client = pool.get().await?;

    let email = form.email;
    // TODO: パスワードを受け取り、それをハッシュ化
    let hashed_password = String::from("aaaa");
    let _ = db::queries::users::create_user()
        .bind(&client, &email.as_str(), &hashed_password.as_str())
        .await?;

    // 303 ユーザーリストにリダイレクト
    Ok(Redirect::to("/").into_response())
}
```

そして私たちは、サーバーに直接リクエストを送信することにより、検証が機能しているかテストすることができます（ブラウザのフォームを回避して）。

```sh
curl http://localhost:3000/sign_up --data-raw 'email=bad-data'
```

## アセットパイプライン

アセットパイプラインは、JavaScriptとCSSアセットを連結そして縮小または圧縮するフレームワークを提供します。
また、それは他の言語と[TypeScript](https://www.typescriptlang.org/)や[SASS](https://sass-lang.com/)のようなプリプロセッサでこれらのアセットを記述する能力を追加します。

私は、いくつかのプロジェクトで[Parcel](https://parceljs.org/)を使用してきましたが、以前は[Webpack](https://webpack.js.org/)でした。
私は、Parcelを使用することが容易であることを発見したので、Nailsでもそれを推奨します。

### ボリュームの準備

もし、あなたが`.devcontainer/docker-compose.yml`を確認した場合、コメントアウトされた行を確認できます。

```yaml
#- node_modules:/workspace/crates/asset-pipeline/node_modules # Set target as a volume for performance.
```

コメントを戻して、開発コンテナをリビルドしてください。
これは、ボリュームとしてnode_moduleフォルダを準備して、ビルドで良いパフォーマンスを提供します。
node_moduleフォルダには多くのファイルがあり、Dockerがそれらをあなたのメインファイルシステムと同期を試みるためです。

また、`.devcontainer/Dockerfile`の次の行をアンコメントしてください。

```Dockerfile
#RUN sudo mkdir -p /workspace/crates/asset-pipeline/node_modules && sudo chown $USERNAME:$USERNAME /workspace/crates/asset-pipeline/node_modules
```

### .gitignore

私たちは、次の`.gitignore`ファイルが必要です。

```text
dist
node_modules
.parcel-cache
```

### Parcelのインストール

Parcelをインストールしてください。

<!-- markdownling-disable MD014 -->
```sh
$ mdir crates/asset-pipeline
$ cd crates/asset-pipeline
$ npm install --save-dev parcel
```
<!-- markdownlint-enable -->

`crates/asset-pipeline/index.ts`ファイルを作成してください。

```typescript
import './scss/index.scss'
```

そして、`crates/asset-pipeline/scss/index.scss`ファイルも作成してください。

```scss
h1 {
    color: red;
}
```

そして、`package.json`のscriptセクションに追加してください。

```json
 "scripts": {
    "start": "parcel watch ./asset-pipeline/index.ts",
    "release": "parcel build ./asset-pipeline/index.ts"
  },
```
