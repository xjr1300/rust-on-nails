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
