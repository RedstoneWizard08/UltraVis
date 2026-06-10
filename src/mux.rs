use crate::{
    parser::{ProgramOptions, TimeSigItem},
    render::{FrameInfo, render_frame},
};
use crossbeam::channel::bounded;
use eyre::Result;
use ffmpeg_next::{format, media::Type};
use indicatif::ProgressIterator;
use rayon::{
    ThreadPoolBuilder,
    iter::{ParallelBridge, ParallelIterator},
};
use std::{collections::BTreeMap, fs, path::PathBuf};
use video_rs::{Time, encode::Settings};

pub const MAX_FRAME_QUEUE: usize = 30;
pub const MAX_THREADS: usize = 30;

pub fn mux_video(times: &[TimeSigItem], opts: &ProgramOptions, out_path: PathBuf) -> Result<()> {
    // 60s / bpm
    let beat_secs = 60.0 / opts.bpm.0;

    let tmp_path =
        out_path.with_extension(format!("tmp.{}", out_path.extension().unwrap().display()));

    let mut frames = Vec::new();
    let mut i = 0;
    let mut ps = Time::zero();

    for (pos, time) in times.iter().enumerate() {
        for m in 0..time.measures {
            let next = if m == time.measures - 1 && pos as usize + 1 < times.len() {
                Some(times[pos + 1])
            } else {
                None
            };

            for beat in 0..time.num {
                let frame = FrameInfo {
                    idx: i,
                    beat,
                    time: *time,
                    next,
                    wait: beat_secs * (8.0 / time.den as f32),
                    pos: ps,
                    width: opts.width,
                    height: opts.height,
                    bpm: opts.bpm.0,
                    note: opts.bpm_divisor,
                };

                ps = ps.aligned_with(Time::from_secs(frame.wait)).add();
                frames.push(frame);

                i += 1;
            }
        }
    }

    let mut enc = video_rs::Encoder::new(
        tmp_path.clone(),
        Settings::preset_h264_yuv420p(opts.width, opts.height, false),
    )?;

    let (tx, rx) = bounded(MAX_FRAME_QUEUE);

    ThreadPoolBuilder::new()
        .num_threads(MAX_THREADS)
        .build_global()?;

    let handle = std::thread::spawn(move || {
        frames
            .into_iter()
            .progress()
            .enumerate()
            .par_bridge()
            .map_with(tx.clone(), |tx, (i, f)| {
                let data = render_frame(f);

                tx.send((i, data)).unwrap();
            })
            .collect::<Vec<_>>();
    });

    let mut buf = BTreeMap::new();
    let mut pos = 0;

    while let Ok((i, data)) = rx.recv() {
        buf.insert(i, data);

        if let Some((info, data)) = buf.remove(&pos) {
            enc.encode(&data, info.pos)?;
            pos += 1;
        }
    }

    for (info, data) in buf.into_values() {
        enc.encode(&data, info.pos)?;
    }

    handle.join().unwrap();
    enc.finish()?;

    if let Some(aux_path) = &opts.song_path {
        ffmpeg_next::init()?;

        let mut vcx = format::input(&tmp_path)?;
        let mut acx = format::input(&aux_path)?;

        let mut out = format::output(&out_path)?;

        let v_stream = vcx.streams().best(Type::Video).unwrap();
        let a_stream = acx.streams().best(Type::Audio).unwrap();

        let vtb = v_stream.time_base();
        let atb = a_stream.time_base();

        let mut v_stream_out = out.add_stream(None)?;

        v_stream_out.set_parameters(v_stream.parameters());
        (unsafe { *v_stream_out.parameters().as_ptr() }).codec_tag = 0;

        let v_idx = v_stream_out.index();

        let mut a_stream_out = out.add_stream(None)?;

        a_stream_out.set_parameters(a_stream.parameters());
        (unsafe { *a_stream_out.parameters().as_ptr() }).codec_tag = 0;

        let a_idx = a_stream_out.index();

        out.write_header()?;

        for (_stream, mut pkt) in vcx.packets() {
            let out_s = out.stream(v_idx).unwrap();

            pkt.rescale_ts(vtb, out_s.time_base());
            pkt.set_position(-1);
            pkt.set_stream(out_s.index());
            pkt.write_interleaved(&mut out).unwrap();
        }

        for (_stream, mut pkt) in acx.packets() {
            let out_s = out.stream(a_idx).unwrap();

            pkt.rescale_ts(atb, out_s.time_base());
            pkt.set_position(-1);
            pkt.set_stream(out_s.index());
            pkt.write_interleaved(&mut out).unwrap();
        }

        out.write_trailer()?;
    } else {
        fs::rename(&tmp_path, &out_path)?;
    }

    Ok(())
}
