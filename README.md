# シンプルメモ帳

GPUI Kit を使った Windows 向けのメモ帳です。
新規作成、UTF-8 ファイルを開く、保存、名前を付けて保存に対応しています。
新規作成・開く・終了ボタンでは未保存の変更を破棄する前に確認します。
ウィンドウの × ボタンには未保存確認を実装していないため、保存後に閉じてください。
Shift-JIS ファイルは対象外です。

## 起動

Rust（MSVC ツールチェーン）と Visual Studio Build Tools の C++ 開発環境が必要です。

```powershell
cd D:\gpui-kit-rust\notepad
cargo run
```

## exe の作成

```powershell
cargo build --release
```

生成先: `target/release/simple-notepad.exe`

この環境では Rust が見つからないため、ビルドと実際の画面操作は未検証です。

## GitHub Actions でビルド

このフォルダーの内容（隠しフォルダー `.github` を含む）を公開リポジトリの
ルートに配置し、`main` ブランチへ push します。
Actions の `Build Windows Notepad` が実行されます。
成功した実行の Artifacts から `simple-notepad-windows` をダウンロードし、
ZIP を展開して `simple-notepad.exe` を起動します。成果物は3日間保管します。
Rust は GitHub 側だけにインストールされます。

GPUI Kit は確認したコミットに固定して取得するため、隣のフォルダーは不要です。
