use crate::parser::{Note, TimeSigItem};
use font_kit::{font::Font, loader::Loader};
use ndarray::Array3;
use raqote::{DrawOptions, DrawTarget, Point, SolidSource, Source};
use std::sync::Arc;
use video_rs::Time;

const FONT: &[u8] = include_bytes!("../res/VCR_OSD_MONO_1.001.ttf");

fn measure_text(font: &Font, size: f32, s: &str) -> (f32, f32) {
    let mut sum = 0.0;
    let upe = font.metrics().units_per_em as f32;

    for glyph in s.chars().filter_map(|it| font.glyph_for_char(it)) {
        if let Ok(adv) = font.advance(glyph) {
            let cw = (adv.x() / upe) * size;

            sum += cw;
        }
    }

    (sum, (font.metrics().bounding_box.height() / upe) * size)
}

pub fn render_frame(info: FrameInfo) -> (FrameInfo, Array3<u8>) {
    let font = Loader::from_bytes(Arc::new(FONT.to_vec()), 0).unwrap();
    let beat = info.beat;
    let next = info.next;

    let font_size = 72.0;
    let cx = info.width as f32 / 2.0;
    let cy = info.height as f32 / 2.0;

    let mut dt = DrawTarget::new(info.width as i32, info.height as i32);

    dt.clear(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0));

    let time = format!("{}/{}", info.time.num, info.time.den);

    let (tw, th) = measure_text(&font, font_size, &time);
    let offset = th / 2.0 + 3.0;
    let x = cx - (tw / 2.0);
    let y = cy - (th / 2.0) - offset;

    dt.draw_text(
        &font,
        font_size,
        &time,
        Point::new(x, y),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)),
        &DrawOptions::new(),
    );

    let full = (1..=info.time.num)
        .map(|it| format!("{it}"))
        .collect::<Vec<_>>()
        .join(" ");

    let size = measure_text(&font, font_size, &full);

    let x = cx - (size.0 / 2.0);
    let y = cy - (size.1 / 2.0) + offset;

    let mut cur_x = x;

    for n in 1..=info.time.num {
        let txt = format!("{n} ");
        let (tw, _) = measure_text(&font, font_size, &txt);

        let color = if n == beat + 1 {
            SolidSource::from_unpremultiplied_argb(255, 255, 0, 0)
        } else {
            SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)
        };

        dt.draw_text(
            &font,
            font_size,
            &txt,
            Point::new(cur_x, y),
            &Source::Solid(color),
            &DrawOptions::new(),
        );

        cur_x += tw;
    }

    if let Some(next) = next {
        let next_txt = format!("Next: {}/{}", next.num, next.den);
        let (tw, th) = measure_text(&font, 40.0, &next_txt);
        let (x, y) = (info.width as f32 - tw, info.height as f32 - th);

        dt.draw_text(
            &font,
            40.0,
            &next_txt,
            Point::new(x, y),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(
                255, 0xF7, 0xD9, 0x6F,
            )),
            &DrawOptions::new(),
        );
    }

    let bpm_txt = format!("{} BPM (8th note)", info.bpm);
    let (_, th) = measure_text(&font, 40.0, &bpm_txt);
    let (x, y) = (0.0, info.height as f32 - th);

    dt.draw_text(
        &font,
        40.0,
        &bpm_txt,
        Point::new(x, y),
        &Source::Solid(SolidSource::from_unpremultiplied_argb(200, 45, 130, 235)),
        &DrawOptions::new(),
    );

    let vec = dt
        .into_vec()
        .into_iter()
        .map(|it| {
            [
                (it >> 16) as u8 & 0xFF_u8,
                (it >> 8) as u8 & 0xFF_u8,
                it as u8 & 0xFF_u8,
                (it >> 24) as u8 & 0xFF_u8,
            ]
        })
        .collect::<Vec<_>>();

    (
        info,
        Array3::from_shape_fn((info.height, info.width, 3), |(y, x, c)| {
            vec[(info.width as usize * y) + x][c]
        }),
    )
}

#[derive(Debug, Clone, Copy)]
pub struct FrameInfo {
    pub idx: usize,
    pub beat: usize,
    pub time: TimeSigItem,
    pub next: Option<TimeSigItem>,

    /// Seconds to wait before the next frame.
    pub wait: f32,
    pub pos: Time,
    pub width: usize,
    pub height: usize,
    pub bpm: f32,
    pub note: Note,
}
