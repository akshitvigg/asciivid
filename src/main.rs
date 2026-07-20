use crossterm::cursor::{Hide, Show};
use crossterm::event::Event::{self};
use crossterm::event::{KeyCode, poll, read};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};
use ffmpeg_next::decoder::Video as VideoDecoder;
use ffmpeg_next::format::Pixel::RGB24;
use ffmpeg_next::format::input;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type::{self};
use ffmpeg_next::software::scaling::{Context, Flags};
use ffmpeg_next::{Error, Rational};
use std::env;
use std::fmt::Write;
use std::io::{Write as IOWrite, stdout};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

fn get_img_path() -> Result<String, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("add image path");
        return Err("add image path".to_string());
    }

    let image_name = &args[1];
    Ok(image_name.to_string())
}

const RAMP: &str = " .,:;irsXA253hMHGS#9B&@";
const MAGIC_NUM: f64 = (RAMP.len() - 1) as f64 / 255.0;

fn brightness_to_ascii(brightness: u16) -> char {
    let ramp_ind = (brightness) as f64 * MAGIC_NUM;
    let ramp_abs = ramp_ind.round();
    let ascii_char = RAMP.as_bytes()[ramp_abs as usize] as char;
    ascii_char
}

fn brightness(r: u8, g: u8, b: u8) -> u16 {
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u16
}

fn rgb_at(frame: &Video, x: usize, y: usize) -> (u8, u8, u8) {
    let rgb_plane = frame.data(0);
    let row = y * frame.stride(0);
    let offset = x * 3;
    let r = rgb_plane[row + offset].to_owned();
    let g = rgb_plane[row + offset + 1].to_owned();
    let b = rgb_plane[row + offset + 2].to_owned();
    (r, g, b)
}

fn open_input(source: &str) -> Result<ffmpeg_next::format::context::Input, ffmpeg_next::Error> {
    let path = Path::new(&source);
    input(path)
}

fn is_youtube_url(input: &str) -> bool {
    if input.starts_with("https://") && input.contains("yout") {
        return true;
    } else {
        return false;
    }
}

fn process_frame(
    decoded: &Video,
    scaler: &mut Context,
    scaled: &mut Video,
    playback_start: Instant,
    total_paused: Duration,
    frame_secs: f64,
) -> Result<(), Error> {
    scaler.run(decoded, scaled)?;

    let mut ascii_frame = String::new();

    // Move cursor to top-left (0,0) before rewriting frame
    // to avoid scrolling lag and screen tearing
    write!(ascii_frame, "\x1b[H").unwrap();

    for y in 0..scaled.height() {
        for x in 0..scaled.width() {
            let (r, g, b) = rgb_at(&scaled, x as usize, y as usize);
            let brightness = brightness(r, g, b);

            let ascii_char = brightness_to_ascii(brightness);
            write!(
                ascii_frame,
                "\x1b[38;2;{};{};{}m{}\x1b[0m",
                r, g, b, ascii_char
            )
            .unwrap();
        }
        ascii_frame.push_str("\r\n");
    }

    let real_elapsed_secs = (Instant::now() - playback_start - total_paused).as_secs_f64();

    if frame_secs > real_elapsed_secs {
        let drift = frame_secs - real_elapsed_secs;
        thread::sleep(Duration::from_secs_f64(drift));
    } else {
        // let lag = real_elapsed_secs - frame_secs;

        // if lag > 0.1 {
        //     continue;
        // }
    }
    print!("{}", ascii_frame);
    stdout().flush().unwrap();
    Ok(())
}

fn pts_to_secs(pts: i64, time_base: Rational) -> f64 {
    let secs = (pts as f64 * f64::from(time_base.0)) / f64::from(time_base.1);
    secs
}

fn drain_decoder(
    decoded: &mut Video,
    decoder: &mut VideoDecoder,
    scaler: &mut Context,
    scaled: &mut Video,
    time_base: Rational,
    state: &mut PlaybackState,
) -> Result<PlaybackAction, Box<dyn std::error::Error>> {
    while decoder.receive_frame(decoded).is_ok() {
        if let Some(pts) = decoded.pts() {
            state.current_pts = pts;
            state.current_time = pts_to_secs(pts, time_base);
        }

        if state.playback_start.is_none() {
            // Anchor the real-time clock to the frame's own timestamp,
            // so it doesn't matter whether this is frame 0 or a post-seek frame.
            state.playback_start =
                Some(Instant::now() - Duration::from_secs_f64(state.current_time));
            state.total_paused = Duration::ZERO;
        }

        let playback_start = state.playback_start.unwrap();

        if poll(Duration::ZERO)? {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char(' ') {
                    if !state.is_paused {
                        state.pause_started = Some(Instant::now());
                        state.is_paused = true;
                    }
                }
                if key.code == KeyCode::Char('q') {
                    return Ok(PlaybackAction::Quit);
                }

                if key.code == KeyCode::Left {
                    if state.current_time - 5.0 <= 0.0 {
                        state.seek_target_secs = 0.0;
                    } else {
                        state.seek_target_secs = state.current_time - 5.0;
                    }
                    return Ok(PlaybackAction::Seek);
                }

                if key.code == KeyCode::Right {
                    state.seek_target_secs = state.current_time + 5.0;
                    return Ok(PlaybackAction::Seek);
                }
            }
        }

        // println!(
        //     "Current: {:.2}s ({})",
        //     state.current_time, state.current_pts
        // );
        //
        // println!(
        //     "Target : {:.2}s ({})",
        //     state.seek_target_secs,
        //     secs_to_timestamp(state.seek_target_secs, time_base)
        // );
        //
        while state.is_paused {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char(' ') {
                    let duration = Instant::now() - state.pause_started.unwrap();
                    state.total_paused += duration;
                    state.pause_started = None;
                    state.is_paused = false;
                }

                if key.code == KeyCode::Char('q') {
                    return Ok(PlaybackAction::Quit);
                }
                if key.code == KeyCode::Left {
                    if state.current_time - 5.0 <= 0.0 {
                        state.seek_target_secs = 0.0;
                    } else {
                        state.seek_target_secs = state.current_time - 5.0;
                    }
                    return Ok(PlaybackAction::Seek);
                }

                if key.code == KeyCode::Right {
                    state.seek_target_secs = state.current_time + 5.0;
                    return Ok(PlaybackAction::Seek);
                }
            }
        }

        process_frame(
            decoded,
            scaler,
            scaled,
            playback_start,
            state.total_paused,
            state.current_time,
        )?;
    }
    Ok(PlaybackAction::Continue)
}

struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> std::io::Result<Self> {
        //clear screen once b4 playing
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Clear(ClearType::All), Hide)?;
        out.flush()?;
        Ok(Self)
    }
}

enum PlaybackAction {
    Continue,
    Seek,
    Quit,
}

struct PlaybackState {
    playback_start: Option<Instant>,
    total_paused: Duration,
    pause_started: Option<Instant>,
    is_paused: bool,
    current_pts: i64,
    current_time: f64,
    seek_target_secs: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TerminalGuard::new()?;
    let (_w, _h) = size()?;
    let pathdemo = get_img_path()?;

    ffmpeg_next::init()?;

    print!("{}", is_youtube_url(&pathdemo));

    let mut ictx = open_input(pathdemo.as_str())?;

    let input_stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg_next::Error::StreamNotFound)?;

    let video_stream_index = input_stream.index();

    let context_decoder =
        ffmpeg_next::codec::context::Context::from_parameters(input_stream.parameters())?;

    let mut decoder = context_decoder.decoder().video()?;

    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        RGB24,
        _w as u32,
        _h as u32,
        Flags::LANCZOS,
    )?;

    let mut scaled = Video::empty();

    let time_base = input_stream.time_base();

    let mut decoded = Video::empty();

    let mut state = PlaybackState {
        playback_start: None,
        total_paused: Duration::ZERO,
        pause_started: None,
        is_paused: false,
        current_pts: 0,
        current_time: 0.0,
        seek_target_secs: 0.0,
    };

    'playback: loop {
        for (stream, packet) in ictx.packets() {
            if stream.index() != video_stream_index {
                continue;
            }

            decoder.send_packet(&packet)?;

            let action = drain_decoder(
                &mut decoded,
                &mut decoder,
                &mut scaler,
                &mut scaled,
                time_base,
                &mut state,
            )?;

            match action {
                PlaybackAction::Continue => {}
                PlaybackAction::Seek => {
                    let seek_time = (state.seek_target_secs * 1_000_000.0) as i64;
                    ictx.seek(seek_time, ..seek_time)?;
                    decoder.flush();
                    state.playback_start = None;
                    continue 'playback;
                }
                PlaybackAction::Quit => return Ok(()),
            }
        }
        decoder.send_eof()?;

        drain_decoder(
            &mut decoded,
            &mut decoder,
            &mut scaler,
            &mut scaled,
            time_base,
            &mut state,
        )?;

        break;
    }
    Ok(())
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
        let mut out = stdout();
        let _ = execute!(out, LeaveAlternateScreen, Show);
        let _ = out.flush();
    }
}
