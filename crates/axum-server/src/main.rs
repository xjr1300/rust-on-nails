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
