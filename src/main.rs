#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::PathBuf;
use gpui_kit::{*, assets::Assets, component::{*, button::*, input::{Editor, EditorState}}};

struct Notepad {
    editor: Entity<EditorState>,
    path: Option<PathBuf>,
    saved: String,
    status: SharedString,
    busy: bool,
}

impl Notepad {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            editor: cx.new(|cx| EditorState::new(window, cx)
                .line_number(true).soft_wrap(true).placeholder("ここにメモを入力してください")),
            path: None,
            saved: String::new(),
            busy: false,
            status: "新しいメモ — UTF-8".into(),
        }
    }

    fn text(&self, cx: &App) -> String {
        self.editor.read(cx).value().to_string()
    }

    fn confirm_discard(&mut self, cx: &App) -> bool {
        if self.text(cx) == self.saved { return true; }
        rfd::MessageDialog::new().set_title("未保存の変更")
            .set_description("保存されていない変更を破棄しますか？")
            .set_buttons(rfd::MessageButtons::YesNo).show() == rfd::MessageDialogResult::Yes
    }

    fn save(&mut self, save_as: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy { return; }
        self.busy = true;
        let existing = if save_as { None } else { self.path.clone() };
        let text = self.text(cx);
        cx.spawn_in(window, async move |view, window| {
            let path = match existing {
                Some(path) => Some(path),
                None => rfd::AsyncFileDialog::new().add_filter("テキスト", &["txt"])
                    .set_file_name("メモ.txt").save_file().await.map(|file| file.path().to_path_buf()),
            };
            let result = if let Some(path) = path {
                Some(smol::unblock(move || {
                    let result = std::fs::write(&path, &text);
                    (path, text, result)
                }).await)
            } else { None };
            let _ = view.update_in(window, |this, _, cx| {
                this.busy = false;
                if let Some((path, text, result)) = result {
                    match result {
                        Ok(()) => {
                            this.status = format!("保存しました: {}", path.display()).into();
                            this.path = Some(path);
                            this.saved = text;
                        }
                        Err(error) => this.status = format!("保存できません: {error}").into(),
                    }
                }
                cx.notify();
            });
        }).detach();
        cx.notify();
    }

    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || !self.confirm_discard(cx) { return; }
        self.busy = true;
        cx.spawn_in(window, async move |view, window| {
            let file = rfd::AsyncFileDialog::new().add_filter("テキスト", &["txt", "md", "log"]).pick_file().await;
            let result = if let Some(file) = file {
                let path = file.path().to_path_buf();
                Some(smol::unblock(move || {
                    let result = std::fs::read_to_string(&path);
                    (path, result)
                }).await)
            } else { None };
            let _ = view.update_in(window, |this, window, cx| {
                this.busy = false;
                if let Some((path, result)) = result { match result {
            Ok(text) => {
                let text = text.trim_start_matches('\u{feff}').to_string();
                this.editor.update(cx, |editor, cx| editor.set_value(text.clone(), window, cx));
                this.saved = text;
                this.status = format!("開きました: {}", path.display()).into();
                this.path = Some(path);
            }
            Err(error) => this.status = format!("開けません（UTF-8 ファイルを選んでください）: {error}").into(),
                }}
                cx.notify();
            });
        }).detach();
        cx.notify();
    }
}

impl Render for Notepad {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().gap_2().p_3()
            .child(h_flex().gap_2()
                .child(Button::new("new").label("新規").on_click(cx.listener(|this, _, window, cx| {
                    if !this.busy && this.confirm_discard(cx) {
                        this.editor.update(cx, |editor, cx| editor.set_value("", window, cx));
                        this.path = None;
                        this.saved.clear();
                        this.status = "新しいメモ — UTF-8".into();
                        cx.notify();
                    }
                })))
                .child(Button::new("open").label("開く").on_click(cx.listener(|this, _, window, cx| this.open(window, cx))))
                .child(Button::new("save").primary().label("保存").on_click(cx.listener(|this, _, window, cx| this.save(false, window, cx))))
                .child(Button::new("save-as").label("名前を付けて保存").on_click(cx.listener(|this, _, window, cx| this.save(true, window, cx))))
                .child(Button::new("quit").label("終了").on_click(cx.listener(|this, _, _, cx| {
                    if !this.busy && this.confirm_discard(cx) { cx.quit(); }
                }))))
            .child(div().flex_1().min_h_0().child(Editor::new(&self.editor).disabled(self.busy).size_full()))
            .child(div().text_sm().child(self.status.clone()))
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let path = std::env::temp_dir().join("simple-notepad-crash.log");
        let _ = std::fs::write(path, format!("{info}\n{}", std::backtrace::Backtrace::force_capture()));
    }));
    gpui_kit::application().with_assets(Assets).run(|cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(900.), px(650.)), cx)),
            ..Default::default()
        }, cx, |window, cx| {
            window.set_window_title("シンプルメモ帳");
            cx.new(|cx| Notepad::new(window, cx))
        }).expect("メモ帳を開けませんでした");
        cx.activate(true);
    });
}
