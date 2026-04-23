use crate::analyse::WorkAnalyse;
use plotters::prelude::*;
use std::{collections::HashMap, path::Path};

pub enum XAxis {
    WordCount,
    DecoCount,
    TokenCount,
}

pub fn plot_result(
    x_axis: &XAxis,
    base_path: &Path,
    ok_results: &HashMap<&str, WorkAnalyse>,
) -> Result<(), Box<dyn std::error::Error>> {
    let font = "Yu Gothic";
    let path = base_path.join("result").join(match x_axis {
        XAxis::WordCount => "wordcount_vs_duration.png",
        XAxis::DecoCount => "notecount_vs_duration.png",
        XAxis::TokenCount => "tokencount_vs_duration.png",
    });
    let root = BitMapBackend::new(&path, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;

    let caption = match x_axis {
        XAxis::WordCount => "文字数に対する処理時間の増加",
        XAxis::DecoCount => "装飾数に対する処理時間の増加",
        XAxis::TokenCount => "トークン数に対する処理時間の増加",
    };
    let mut chart = ChartBuilder::on(&root)
        .caption(caption, (font, 30).into_font())
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(
            0f32..match x_axis {
                XAxis::WordCount => 120_0000f32,
                XAxis::DecoCount => 10_0000f32,
                XAxis::TokenCount => 20_0000f32,
            },
            0f32..5f32,
        )?;

    chart.configure_mesh().draw()?;

    chart.draw_series(PointSeries::of_element(
        ok_results.iter().map(|(_, ok)| {
            let text = if ok.pure_parsetime().as_secs_f32() > 0.3 {
                ok.title.clone()
            } else {
                String::new()
            };
            let coords = (
                match x_axis {
                    XAxis::WordCount => ok.word_count,
                    XAxis::DecoCount => ok.deco_count,
                    XAxis::TokenCount => ok.token_count,
                } as f32,
                ok.pure_parsetime().as_secs_f32(),
            );
            (coords, text)
        }),
        6,
        RED.mix(0.5).filled(),
        &|(coord, title), size, style| {
            EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
                + Text::new(title, (0, 15), (font, 15).into_font())
        },
    ))?;

    root.present()?;

    Ok(())
}
