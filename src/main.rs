#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::PathBuf;
use gpui_kit::{*, assets::Assets, component::{*, button::*, input::{Editor, EditorState}}};

struct Notepad {
    editor: Entity<EditorState>,
    path: Option<PathBuf>,
    saved: String,
    status: SharedString,
}

impl Notepad {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            editor: cx.new(|cx| EditorState::new(window, cx)
                .line_number(true).soft_wrap(true).placeholder("ここにメモを入力してください")),
            path: None,
            saved: String::new(),
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

    fn save(&mut self, save_as: bool, cx: &mut Context<Self>) {
        let path = if save_as || self.path.is_none() {
            rfd::FileDialog::new().add_filter("テキスト", &["txt"])
                .set_file_name("メモ.txt").save_file()
        } else { self.path.clone() };
        let Some(path) = path else { return; };
        let text = self.text(cx);
        match std::fs::write(&path, &text) {
            Ok(()) => {
                self.status = format!("保存しました: {}", path.display()).into();
                self.path = Some(path);
                self.saved = text;
            }
            Err(error) => self.status = format!("保存できません: {error}").into(),
        }
        cx.notify();
    }

    fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.confirm_discard(cx) { return; }
        let Some(path) = rfd::FileDialog::new().add_filter("テキスト", &["txt", "md", "log"]).pick_file() else { return; };
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let text = text.trim_start_matches('\u{feff}').to_string();
                self.editor.update(cx, |editor, cx| editor.set_value(text.clone(), window, cx));
                self.saved = text;
                self.status = format!("開きました: {}", path.display()).into();
                self.path = Some(path);
            }
            Err(error) => self.status = format!("開けません（UTF-8 ファイルを選んでください）: {error}").into(),
        }
        cx.notify();
    }
}

impl Render for Notepad {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().size_full().gap_2().p_3()
            .child(h_flex().gap_2()
                .child(Button::new("new").label("新規").on_click(cx.listener(|this, _, window, cx| {
                    if this.confirm_discard(cx) {
                        this.editor.update(cx, |editor, cx| editor.set_value("", window, cx));
                        this.path = None;
                        this.saved.clear();
                        this.status = "新しいメモ — UTF-8".into();
                        cx.notify();
                    }
                })))
                .child(Button::new("open").label("開く").on_click(cx.listener(|this, _, window, cx| this.open(window, cx))))
                .child(Button::new("save").primary().label("保存").on_click(cx.listener(|this, _, _, cx| this.save(false, cx))))
                .child(Button::new("save-as").label("名前を付けて保存").on_click(cx.listener(|this, _, _, cx| this.save(true, cx))))
                .child(Button::new("quit").label("終了").on_click(cx.listener(|this, _, _, cx| {
                    if this.confirm_discard(cx) { cx.quit(); }
                }))))
            .child(div().flex_1().min_h_0().child(Editor::new(&self.editor).size_full()))
            .child(div().text_sm().child(self.status.clone()))
    }
}

fn main() {
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
