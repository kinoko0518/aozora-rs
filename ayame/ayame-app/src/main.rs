#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{f32, fs::File, path::Path, sync::Arc};

use aozora_rs_zip::ImgExtension;
use ayame_core::{AbstractAozoraZip, AozoraHyle, Encoding, PotentialCSS, WritingDirection};
use gpui::{
    App, Application, Bounds, Context, Div, FontWeight, Image, ImageFormat, ImageSource, Window,
    WindowBounds, WindowOptions, actions, div, img, prelude::*, px, rgb, rgba, size,
};
use gpui_component::{
    Icon, StyledExt,
    button::{Button, ButtonCustomVariant, ButtonVariants, DropdownButton},
    checkbox::Checkbox,
    scroll::ScrollableElement,
    switch::Switch,
};
use rfd::FileDialog;

actions!(ayame, [SelectShiftJIS, SelectUtf8]);

struct AyameApp {
    aaz: Option<AbstractAozoraZip>,
    cover: Option<(Vec<u8>, ImgExtension)>,
    writing_direction: WritingDirection,
    encoding: Encoding,
    use_prelude: bool,
    use_miyabi: bool,
}

fn img_ext_to_img_fmt(img_ext: ImgExtension) -> ImageFormat {
    match img_ext {
        ImgExtension::Gif => ImageFormat::Gif,
        ImgExtension::Jpeg => ImageFormat::Jpeg,
        ImgExtension::Png => ImageFormat::Png,
        ImgExtension::Svg => ImageFormat::Svg,
    }
}

impl Default for AyameApp {
    fn default() -> Self {
        Self {
            aaz: None,
            cover: None,
            writing_direction: WritingDirection::Vertical,
            encoding: Encoding::ShiftJIS,
            use_prelude: true,
            use_miyabi: true,
        }
    }
}

fn is_zip(path: &Path) -> bool {
    path.extension().map(|ext| ext == "zip").unwrap_or(false)
}

impl AyameApp {
    // Colours
    const OUTLINE_COLOUR: u32 = 0x334155;
    const BUTTON_BACKGROUND: u32 = 0x1e2b4d;
    const LIGHT_TEXT_COLOUR: u32 = 0x90a1b9;

    // Sizes
    const VERTICALIZE_THRESHOLD: f32 = 600.;

    fn get_meta(&self) -> (String, String) {
        self.aaz
            .as_ref()
            .and_then(|s| s.scan_meta().ok())
            .map(|meta| (meta.title.into(), meta.author.into()))
            .unwrap_or(("作品未選択".into(), "作品未選択".into()))
    }

    fn render_save_and_load(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let (title, author) = self.get_meta();
        let btn_colour = ButtonCustomVariant::new(cx)
            .color(rgb(Self::BUTTON_BACKGROUND).into())
            .border(rgb(Self::OUTLINE_COLOUR).into());
        let save_and_load_height = 40.;
        let load_btn = Button::new("file-select-button")
            .icon(Icon::default().path("icons/load.svg"))
            .h(px(save_and_load_height))
            .flex_1()
            .label("ファイルを選択")
            .custom(btn_colour.clone())
            .on_click(cx.listener(|view, _, _, cx| {
                if let Some(picked) = FileDialog::new()
                    .set_title("変換したいファイルを選択してください")
                    .add_filter("変換対象", &["txt", "zip"])
                    .pick_file()
                {
                    let read = std::fs::read(&picked).unwrap();
                    let hyle = if is_zip(&picked) {
                        AozoraHyle::Zip((read, view.encoding))
                    } else {
                        AozoraHyle::Txt((read, view.encoding))
                    };
                    view.aaz = hyle.try_into().ok();
                    cx.notify();
                }
            }));
        let save_btn = Button::new("save-button")
            .icon(Icon::default().path("icons/download.svg"))
            .h(px(save_and_load_height))
            .flex_1()
            .label("保存する")
            .custom(btn_colour)
            .on_click(cx.listener(move |view, _, _, _| {
                if let (Some(save_to), Some(picked)) = (
                    FileDialog::new()
                        .add_filter("EPUB", &["epub"])
                        .set_file_name(format!("[{}] {}", author, title))
                        .save_file(),
                    view.aaz.clone(),
                ) {
                    let zip = File::create(save_to).unwrap();
                    picked
                        .generate_epub(
                            zip,
                            PotentialCSS {
                                use_miyabi: view.use_miyabi,
                                use_prelude: view.use_prelude,
                                direction: view.writing_direction,
                            },
                            "ja",
                        )
                        .unwrap();
                }
            }));

        div()
            .flex()
            .flex_row()
            .w_full()
            .gap_3()
            .child(load_btn)
            .child(save_btn)
    }

    fn render_title_and_author(&mut self) -> impl IntoElement {
        let (title, author) = self.get_meta();
        let title_and_author = div()
            .flex()
            .flex_col()
            .justify_center()
            .w_full()
            .max_w(px(300.))
            .min_w(px(200.))
            .p_4()
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(0xffffff))
                    .font_weight(FontWeight::EXTRA_BOLD)
                    .child(title.clone()),
            )
            .child(
                div()
                    .text_color(rgb(Self::LIGHT_TEXT_COLOUR))
                    .child(author.clone()),
            );
        title_and_author
    }

    fn render_cover(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let cover_base = div()
            .bg(rgb(0x121a2b))
            .rounded(px(15.))
            .w(px(210.))
            .h(px(297.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .overflow_hidden()
            .gap_2();

        let select_btn = Button::new("select-cover")
            .icon(Icon::default().path("icons/load.svg"))
            .w(px(60.))
            .h(px(60.))
            .rounded(px(f32::MAX))
            .on_click(cx.listener(|view, _, _, _| {
                if let Some(picked) = FileDialog::new()
                    .add_filter("画像ファイル", &["png", "jpeg", "jpg", "gif", "svg"])
                    .pick_file()
                {
                    view.cover = Some((
                        std::fs::read(&picked).unwrap(),
                        match picked
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("")
                        {
                            "png" => ImgExtension::Png,
                            "jpeg" | "jpg" => ImgExtension::Jpeg,
                            "gif" => ImgExtension::Gif,
                            "svg" => ImgExtension::Svg,
                            _ => unreachable!(),
                        },
                    ));
                }
            }));

        let cover = if let Some((data, ext)) = self.cover.as_ref() {
            let image = ImageSource::Image(Arc::new(Image::from_bytes(
                img_ext_to_img_fmt(*ext),
                data.clone(),
            )));
            cover_base.child(img(image).h(px(297.)).rounded(px(15.)))
        } else {
            cover_base
                .border_2()
                .border_color(rgb(Self::OUTLINE_COLOUR))
                .border_dashed()
                .child(select_btn.bg(rgb(Self::BUTTON_BACKGROUND)))
                .child(
                    div()
                        .text_color(rgb(Self::LIGHT_TEXT_COLOUR))
                        .child("表紙を選択"),
                )
        };

        cover
    }

    fn main_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .min_w(px(450.))
            .max_w(px(Self::VERTICALIZE_THRESHOLD))
            .flex()
            .flex_col()
            .justify_center()
            .gap_6()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_around()
                    .items_center()
                    .w_full()
                    .gap_4()
                    .child(self.render_title_and_author())
                    .child(self.render_cover(cx)),
            )
            .child(div().w_full().h(px(1.)).bg(rgba(0x33415560)))
            .child(self.render_save_and_load(cx))
    }

    fn setting_island() -> Div {
        div()
            .w_full()
            .bg(rgb(0x16203a))
            .rounded(px(10.))
            .paddings(px(15.))
            .border_1()
            .flex()
            .flex_col()
            .gap_4()
            .border_color(rgb(Self::OUTLINE_COLOUR))
    }

    fn right_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let write_direction_island = Self::setting_island().child(
            Switch::new("is_vertical")
                .text_color(rgb(0xffffff))
                .label("縦書きにする")
                .checked(matches!(self.writing_direction, WritingDirection::Vertical))
                .on_click(cx.listener(|view, checked, _, cx| {
                    view.writing_direction = if *checked {
                        WritingDirection::Vertical
                    } else {
                        WritingDirection::Horizontal
                    };
                    cx.notify();
                })),
        );
        let css_island = Self::setting_island()
            .child(
                Checkbox::new("use_miyabi")
                    .label("miyabiを無効化")
                    .checked(!self.use_miyabi)
                    .on_click(cx.listener(|view, checked: &bool, _, cx| {
                        view.use_miyabi = !*checked;
                        cx.notify();
                    })),
            )
            .child(
                Checkbox::new("use_prelude")
                    .label("preludeを無効化")
                    .checked(!self.use_prelude)
                    .on_click(cx.listener(|view, checked: &bool, _, cx| {
                        view.use_prelude = !*checked;
                        cx.notify();
                    })),
            );
        let encoding_island = Self::setting_island()
            .text_color(rgb(0xffffff))
            .child("文字コード")
            .child(
                DropdownButton::new("encoding")
                    .button(Button::new("btn_encoding").label(match self.encoding {
                        Encoding::ShiftJIS => "Shift-JIS",
                        Encoding::Utf8 => "UTF-8",
                    }))
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("Shift-JIS", Box::new(SelectShiftJIS))
                            .menu("UTF-8", Box::new(SelectUtf8))
                    }),
            );

        div()
            .flex()
            .flex_col()
            .gap_2()
            .flex_1()
            .min_w(px(250.))
            .max_w(px(Self::VERTICALIZE_THRESHOLD))
            .child(write_direction_island)
            .child(css_island)
            .child(encoding_island)
    }
}

impl Render for AyameApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x0a101e))
            .overflow_y_scrollbar()
            .on_action(cx.listener(|view: &mut Self, _: &SelectShiftJIS, _, cx| {
                view.encoding = Encoding::ShiftJIS;
                cx.notify();
            }))
            .on_action(cx.listener(|view: &mut Self, _: &SelectUtf8, _, cx| {
                view.encoding = Encoding::Utf8;
                cx.notify();
            }))
            .child(
                div()
                    .min_h_full()
                    .w_full()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .justify_center()
                    .items_center()
                    .content_center()
                    .gap_8()
                    .p_8()
                    .child(self.main_panel(cx))
                    .child(self.right_panel(cx)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1000.), px(500.0)), cx);
        gpui_component::init(cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AyameApp::default()),
        )
        .unwrap();
    });
}
