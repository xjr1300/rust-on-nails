# データベース

## データベース

### なぜPostgresか?

* PostgreSQLは、SQL:2016の主要な機能のほとんどをサポートしています。完全な準拠に必要な177のコアな必須機能のうち、PostgreSQLは少なくとも170に準拠しています。加えて、サポートされた任意の機能の長いリストがあります。それは記述しているときに価値はないかもしれませんが、コアSQL:2016への完全な準拠を主張するデータベース管理システムの現行バージョンはありません。
* 垂直または水平スケールする[Citus](https://www.citusdata.com/)のようなツールがあります。
* Postgresは、データベースレベルで認証を実装することを可能にするRLS(Row Level Security)をサポートしています。
* 市販されているハードウェア上で1秒間に1000より多いトランザクションをサポートしています。
* NoSQLをサポートしています。PostgresはJSONや非構造データの他の型を蓄積または検索できます。
* Postgresは、一貫性のあるパフォーマンスの提供と革新的なソリューションを背後に、実績のあるアーキテクチャ、信頼性、データの整合性、堅牢な機能セット、拡張性及びオープンソースコミュニティの献身的さについて、高い評価を得ています。

### Postgresインストールのテスト

Postgresはあなたの`devcontainer`に事前インストールされています。
次を実行しましょう。

<!-- markdownlint-disable MD014 -->
```sh
> psql $DATABASE_URL

psql (14.2 (Debian 14.2-1.pgdg110+1), server 14.1 (Debian 14.1-1.pgdg110+1))
Type "help" for help.

postgres=# \dt
Did not find any relations.
postgres=# \q
```
<!-- markdownlint-enable -->

## データベースマイグレーション

[DBMate](https://github.com/amacneil/dbmate)は、複数の開発者とプロダクションサーバー間でデータベーススキーマを同期を保つデータベースマイグレーションツールです。
私たちは、`devcontaier`内に事前インストールしています。

マイグレーションフォルダを準備した後で、ユーザーのマイグレーションを作成します。

### マイグレーションの作成

<!-- markdownlint-disable MD014 -->
```sh
$ dbmate new user_tables
Creating migration: crates/db/migrations/20220330110026_user_tables.sql
```
<!-- markdownlint-enable -->

あなたが生成したSQLファイルを編集して、次を追加します。

```sql
-- migrate:up

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR NOT NULL UNIQUE,
    hashed_password VARCHAR NOT NULL,
    reset_password_selector VARCHAR,
    reset_password_sent_at TIMESTAMP,
    reset_password_validator_hash VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO users(email, hashed_password) VALUES('test1@test1.com', 'aasdsaddasad');
INSERT INTO users(email, hashed_password) VALUES('test2@test1.com', 'aasdsaddasad');
INSERT INTO users(email, hashed_password) VALUES('test3@test1.com', 'aasdsaddasad');

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    session_verifier VARCHAR NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    otp_code_encrypted VARCHAR NOT NULL,
    otp_code_attempts INTEGER NOT NULL DEFAULT 0,
    otp_code_confirmed BOOLEAN NOT NULL DEFAULT false,
    otp_code_sent BOOLEAN NOT NULL DEFAULT false
);

COMMENT ON TABLE sessions IS 'The users login sessions';
COMMENT ON COLUMN sessions.session_verifier IS ' The session is a 32 byte random number stored in their cookie. This is the sha256 hash of that value.';
COMMENT ON COLUMN sessions.otp_code_encrypted IS 'A 6 digit code that is encrypted here to prevent attackers with read access to the database being able to use it.';
COMMENT ON COLUMN sessions.otp_code_attempts IS 'We count OTP attempts to prevent brute forcing.';
COMMENT ON COLUMN sessions.otp_code_confirmed IS 'Once the user enters the correct value this gets set to true.';
COMMENT ON COLUMN sessions.otp_code_sent IS 'Have we sent the OTP code?';


-- migrate:down
DROP TABLE users;
DROP TABLE sessions;
```

### マイグレーションの実行

マイグレーションをリストして、どれが実行されたかを確認します。

```sh
$ dbmate status
[ ] 20220330110026_user_tables.sql

Applied: 0
Pending: 1
```

新しいマイグレーションを実行します。

```sh
$ dbmate up
Applying: 20220330110026_user_tables.sql
```

そして、動作したか確認します。

```sh
$ psql $DATABASE_URL -c 'SELECT count(*) FROM users;'
 count
-------
      0
(1 row)
```

あなたのプロジェクトフォルダは、現在このようになっているはずです。

```text
├── .devcontainer/
│   └── ...
└── crates/
│         axum-server/
│         │  └── main.rs
│         └── Cargo.toml
│         db/
│         ├── migrations
│         │   └── 20220330110026_user_tables.sql
│         └── schema.sql
├── Cargo.toml
└── Cargo.lock
```

> `./crates/db/schema.sql`はありませんでした。

## データベースアクセス

[Cornucopia](https://github.com/cornucopia-rs/cornucopia)は、SQLの小さなスニペットを持ち、それらをRustの関数に変更するコードジェネレータです。

私たちは、すべてのデータベースロジックを一箇所に保管できるように、`crates/db`フォルダをクレートに変換します。

次を実行します。

<!-- markdownlint-disable MD014 -->
```sh
$ cargo init --lib crates/db
Created library package
```
<!-- markdownlint-enable MD014 -->

### インストール

`crates/db`フォルダに`cd`して、`cornucopia`をプロジェクトにインストールします。

```sh
cd crates/db
cargo add cornucopia_async
```

### SQL定義の作成

`db/queries`フォルダ内に、`users.sql`ファイルを作成して、次の内容を追加します。

```sql
--: User()

--! get_users : User
SELECT
    id,
    email
FROM users;
```

Cornucopiaは、データベースにアクセスするために`get_users`と呼ばれるRust関数を生成するために上記の定義を使用します。
Cornucopiaは、コードを生成するときにPostgresにクエリを確認することに注意してください。

### build.rsの更新

`crates/db/build.rs`ファイルを作成して、次の内容を追加します。
このファイルは、私たちの.sqlファイルが変更されたとき、.sqlファイルをコンパイルしてRustコードにします。

```rust
use std::env;
use std::path::Path;

fn main() {
    // SQLをコンパイル
    cornucopia();
}

fn cornucopia() {
    // For the sake of simplicity, this example uses the defaults.
    // 単純にするために、この例はデフォルトを使用
    let queries_path = "queries";

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let file_path = Path::new(&out_dir).join("cornucopia.rs");

    let db_url = env::var_os("DATABASE_URL").unwrap();

    // もしクエリまたはマイグレーションが変更された場合、このビルドスクリプトを再実行
    println!("cargo:rerun-if-changed={queries_path}");

    // cornucopiaの呼び出し: 必要に応じてCLIコマンドを使用
    let output = std::process::Command::new("cornucopia")
        .arg("-q")
        .arg(queries_path)
        .arg("--serialize")
        .arg("-d")
        .arg(&file_path)
        .arg("live")
        .arg(db_url)
        .output()
        .unwrap();

    // Cornucopiaが適切に動作しない場合、エラーを表示することを試行
    if !output.status.success() {
        panic!("{}", &std::str::from_utf8(&output.stderr).unwrap());
    }
}
```

### コネクションをプーリングする関数の追加

コネクションをプーリングするために、`DATABASE_URL`環境変数をcornucopiaが使用できる何かに変換するために使用する`crates/db/src/lib.rs`に次のコードを追加します。

```rust
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;

pub use cornucopia_async::Params;
pub use deadpool_postgres::{Pool, PoolError, Transaction};
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::ServerName;
pub use tokio_postgres::Error as TokioPostgresError;

pub use queries::users::User;

pub fn create_pool(database_url: &str) -> deadpool_postgres::Pool {
    let config = tokio_postgres::Config::from_str(database_url).unwrap();

    let manager = if config.get_ssl_mode() != tokio_postgres::config::SslMode::Disable {
        let tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(DummyTlsVerifier))
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(tls_config);
        deadpool_postgres::Manager::new(config, tls)
    } else {
        deadpool_postgres::Manager::new(config, tokio_postgres::NoTls)
    };

    deadpool_postgres::Pool::builder(manager).build().unwrap()
}

struct DummyTlsVerifier;

impl ServerCertVerifier for DummyTlsVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

include!(concat!(env!("OUT_DIR"), "/cornucopia.rs"));
```

### フォルダ構成

現在フォルダ構成はこのようになっているはずです。

```text
.
├── .devcontainer/
│   └── ...
└── crates/
│         axum-server/
│         │  └── main.rs
│         └── Cargo.toml
│         db/
│         ├── migrations
│         │   └── 20220330110026_user_tables.sql
│         ├── queries
│         │   └── users.sql
│         ├── src
│         │   └── lib.rs
│         └── build.rs
├── Cargo.toml
└── Cargo.lock
```

### databaseクレートのテスト

`crates/db`フォルダにいることを確認してください。
最初に私たちのプロジェクトにクライアント側の依存を追加します。

```sh
cargo add tokio-postgres
cargo add deadpool-postgres@0.10.5
cargo add tokio-postgres-rustls
cargo add postgres-types
cargo add tokio --features macros,rt-multi-thread
cargo add rustls --features dangerous_configuration
cargo add webpki-roots
cargo add futures
cargo add serde --features derive
```

すべてがビルドできることを確認します。

```sh
cargo build
```

`crates/db/src/lib.rs`の下部に次のコードを追加してください。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn load_users() {

        let db_url = std::env::var("DATABASE_URL").unwrap();
        let pool = create_pool(&db_url);

        let client = pool.get().await.unwrap();

        let users = crate::queries::users::get_users()
            .bind(&client)
            .all()
            .await
            .unwrap();

        dbg!(users);
    }
}
```

`cargo test -- --nocapture`を実行すると、次を確認できます。

```text
Running unittests src/lib.rs (/workspace/target/debug/deps/db-1a59f4c51c8578ce)

running 1 test
[crates/db/src/lib.rs:56] users = [
    User {
        id: 1,
        email: "test1@test1.com",
    },
    User {
        id: 2,
        email: "test2@test1.com",
    },
    User {
        id: 3,
        email: "test3@test1.com",
    },
]

test tests::load_users ... ok
```
