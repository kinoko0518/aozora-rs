use crate::{RESULT_OUT_PATH, analyse::WorkAnalyse};
use image::{Pixel, Rgba, RgbaImage};
use imageproc::drawing::{draw_line_segment_mut, draw_text_mut};
use rusttype::{Font, Scale};
use std::collections::HashMap;

pub enum XAxis {
    WordCount,
    DecoCount,
    TokenCount,
}

pub fn plot_result(
    x_axis: &XAxis,
    ok_results: &HashMap<&str, WorkAnalyse>,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(RESULT_OUT_PATH)?;

    let font_data = include_bytes!("../../NotoSansCJKjp-Regular.otf");
    let font =
        Font::try_from_bytes(font_data as &[u8]).ok_or("フォントの読み込みに失敗しました")?;

    let width = 1920;
    let height = 1080;
    let mut image = RgbaImage::from_pixel(width, height, Rgba([255, 255, 255, 255]));

    let margin_left = 150.0;
    let margin_right = 50.0;
    let margin_top = 100.0;
    let margin_bottom = 100.0;

    let plot_width = width as f32 - margin_left - margin_right;
    let plot_height = height as f32 - margin_top - margin_bottom;

    let (max_x, step_x) = match x_axis {
        XAxis::WordCount => (120_0000f32, 20_0000f32),
        XAxis::DecoCount => (10_0000f32, 2_0000f32),
        XAxis::TokenCount => (20_0000f32, 5_0000f32),
    };
    let max_y = 5f32;
    let step_y = 1f32;

    let scale_x = |x: f32| -> f32 { margin_left + (x / max_x) * plot_width };
    let scale_y = |y: f32| -> f32 { height as f32 - margin_bottom - (y / max_y) * plot_height };

    let color_axes = Rgba([0, 0, 0, 255]);
    let color_grid = Rgba([220, 220, 220, 255]);
    let color_points = Rgba([255, 0, 0, 128]);
    let color_text = Rgba([0, 0, 0, 255]);

    let mut y_val = 0f32;
    while y_val <= max_y {
        let py = scale_y(y_val);
        draw_line_segment_mut(
            &mut image,
            (margin_left, py),
            (width as f32 - margin_right, py),
            color_grid,
        );
        let label = format!("{:.1}", y_val);
        draw_text_mut(
            &mut image,
            color_text,
            (margin_left - 50.0) as i32,
            (py - 10.0) as i32,
            Scale::uniform(20.0),
            &font,
            &label,
        );
        y_val += step_y;
    }

    let mut x_val = 0f32;
    while x_val <= max_x {
        let px = scale_x(x_val);
        draw_line_segment_mut(
            &mut image,
            (px, margin_top),
            (px, height as f32 - margin_bottom),
            color_grid,
        );
        let label = format!("{}", x_val as i32);
        draw_text_mut(
            &mut image,
            color_text,
            (px - 20.0) as i32,
            (height as f32 - margin_bottom + 15.0) as i32,
            Scale::uniform(20.0),
            &font,
            &label,
        );
        x_val += step_x;
    }

    draw_line_segment_mut(
        &mut image,
        (margin_left, margin_top),
        (margin_left, height as f32 - margin_bottom),
        color_axes,
    );
    draw_line_segment_mut(
        &mut image,
        (margin_left, height as f32 - margin_bottom),
        (width as f32 - margin_right, height as f32 - margin_bottom),
        color_axes,
    );

    let caption = match x_axis {
        XAxis::WordCount => "文字数に対する処理時間の増加",
        XAxis::DecoCount => "装飾数に対する処理時間の増加",
        XAxis::TokenCount => "トークン数に対する処理時間の増加",
    };
    draw_text_mut(
        &mut image,
        color_text,
        (width / 2 - 200) as i32,
        30,
        Scale::uniform(40.0),
        &font,
        caption,
    );

    let radius = 8i32;
    for (_, ok) in ok_results {
        let x_val = match x_axis {
            XAxis::WordCount => ok.word_count as f32,
            XAxis::DecoCount => ok.deco_count as f32,
            XAxis::TokenCount => ok.token_count as f32,
        };
        let y_val = ok.pure_parsetime().as_secs_f32();

        if x_val > max_x || y_val > max_y {
            continue;
        }

        let cx = scale_x(x_val) as i32;
        let cy = scale_y(y_val) as i32;

        for y in (cy - radius)..=(cy + radius) {
            for x in (cx - radius)..=(cx + radius) {
                if (x - cx).pow(2) + (y - cy).pow(2) <= radius.pow(2) {
                    if let Some(pixel) = image.get_pixel_mut_checked(x as u32, y as u32) {
                        pixel.blend(&color_points);
                    }
                }
            }
        }

        if y_val > 0.3 {
            draw_text_mut(
                &mut image,
                color_text,
                cx + 8,
                cy - 12,
                Scale::uniform(20.0),
                &font,
                &ok.title,
            );
        }
    }

    let path = format!(
        "{}/{}",
        RESULT_OUT_PATH,
        match x_axis {
            XAxis::WordCount => "wordcount_vs_duration.png",
            XAxis::DecoCount => "notecount_vs_duration.png",
            XAxis::TokenCount => "tokencount_vs_duration.png",
        }
    );

    image.save(&path)?;

    Ok(())
}
