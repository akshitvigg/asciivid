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
    (r as u16 + g as u16 + b as u16) / 3
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

fn process_frame(
    decoded: &Video,
    scaler: &mut Context,
    scaled: &mut Video,
    time_base: Rational,
    playback_start: Instant,
    total_paused: Duration,
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
    let pts_opts = decoded.pts();

    if let Some(pts) = pts_opts {
        let frame_secs = (pts as f64 * f64::from(time_base.0)) / f64::from(time_base.1);

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
    }
    print!("{}", ascii_frame);
    stdout().flush().unwrap();
    Ok(())
}

fn drain_decoder(
    decoded: &mut Video,
    decoder: &mut VideoDecoder,
    scaler: &mut Context,
    scaled: &mut Video,
    time_base: Rational,
    state: &mut PlaybackState,
) -> Result<(), Box<dyn std::error::Error>> {
    while decoder.receive_frame(decoded).is_ok() {
        if state.playback_start.is_none() {
            state.playback_start = Some(Instant::now());
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
                    state.should_quit = true;
                }
            }
        }

        while state.is_paused {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char(' ') {
                    let duration = Instant::now() - state.pause_started.unwrap();
                    state.total_paused += duration;
                    state.pause_started = None;
                    state.is_paused = false;
                }

                if key.code == KeyCode::Char('q') {
                    state.should_quit = true;
                    return Ok(());
                }
            }
        }

        process_frame(
            decoded,
            scaler,
            scaled,
            time_base,
            playback_start,
            state.total_paused,
        )?;

        if state.should_quit {
            return Ok(());
        }
    }
    Ok(())
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

struct PlaybackState {
    playback_start: Option<Instant>,
    total_paused: Duration,
    pause_started: Option<Instant>,
    is_paused: bool,
    should_quit: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = TerminalGuard::new()?;
    let (_w, _h) = size()?;
    let pathdemo = get_img_path()?;

    ffmpeg_next::init()?;

    let path = Path::new(&pathdemo);

    let mut ictx = input(path)?;

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
        Flags::BILINEAR,
    )?;

    let mut scaled = Video::empty();

    let time_base = input_stream.time_base();

    let mut decoded = Video::empty();

    let mut state = PlaybackState {
        playback_start: None,
        total_paused: Duration::ZERO,
        pause_started: None,
        is_paused: false,
        should_quit: false,
    };

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            drain_decoder(
                &mut decoded,
                &mut decoder,
                &mut scaler,
                &mut scaled,
                time_base,
                &mut state,
            )?;

            if state.should_quit {
                break;
            }
        }
    }

    if !state.should_quit {
        decoder.send_eof()?;

        drain_decoder(
            &mut decoded,
            &mut decoder,
            &mut scaler,
            &mut scaled,
            time_base,
            &mut state,
        )?;
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
