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
