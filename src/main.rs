use crossterm::terminal::size;
use ffmpeg_next::format::Pixel::RGB24;
use ffmpeg_next::format::input;
use ffmpeg_next::frame::Video;
use ffmpeg_next::media::Type::{self};
use ffmpeg_next::software::scaling::{Context, Flags};
use image::imageops::FilterType::Nearest;
use image::{DynamicImage, GenericImageView, ImageError};
use std::env;
use std::fmt::Write;
use std::path::Path;
use std::time::Instant;

fn get_img_path() -> Result<String, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("add image path");
        return Err("add image path".to_string());
    }

    let image_name = &args[1];
    Ok(image_name.to_string())
}

fn load_image(path: &Path) -> Result<DynamicImage, ImageError> {
    image::ImageReader::open(path)?.decode()
}

fn resize_image(img: DynamicImage, w: u16, h: u16) -> DynamicImage {
    image::DynamicImage::resize_exact(&img, w as u32, h as u32, Nearest)
}

const RAMP: &str = " .,:;irsXA253hMHGS#9B&@";
const MAGIC_NUM: f64 = (RAMP.len() - 1) as f64 / 255.0;

fn brightness_to_ascii(brightness: u16) -> char {
    let ramp_ind = (brightness) as f64 * MAGIC_NUM;
    let ramp_abs = ramp_ind.round();
    let ascii_char = RAMP.as_bytes()[ramp_abs as usize] as char;
    ascii_char
}

fn image_to_ascii(img: &DynamicImage) -> String {
    let mut prev_y = 0;

    let mut frame = String::new();

    for (_, y, pixel) in img.pixels() {
        let brightness = (pixel[0] as u16 + pixel[1] as u16 + pixel[2] as u16) / 3;

        if prev_y != y {
            frame.push('\n');
        }
        let ascii_char = format!(
            "\x1b[38;2;{};{};{}m{}\x1b[0m",
            pixel[0],
            pixel[1],
            pixel[2],
            brightness_to_ascii(brightness)
        );
        frame.push_str(ascii_char.as_str());

        prev_y = y;
    }
    frame
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_w, _h) = size()?;
    let pathdemo = get_img_path()?;
    //
    // let img = load_image(&path)?;
    // let resized_img = resize_image(img, w, h);
    //
    // let mut frames: Vec<PathBuf> = fs::read_dir("/home/akira/marchFrames/")?
    //     .map(|e| e.unwrap().path())
    //     .collect();
    //
    // frames.sort();
    //
    // let mut ascii_frames: Vec<String> = Vec::new();
    // let start = Instant::now();
    // println!("started processing");
    //
    // for frame_path in frames {
    //     let frame_start = Instant::now();
    //     let img = load_image(&frame_path)?;
    //     print!("load={:?} ", frame_start.elapsed());
    //
    //     let resize_start = Instant::now();
    //     let resized_img = resize_image(img, w, h);
    //     print!(
    //         "resize={:?} ,img_dimension = {:?} ",
    //         resize_start.elapsed(),
    //         resized_img.dimensions()
    //     );
    //
    //     let frame_start = Instant::now();
    //     let frame = image_to_ascii(&resized_img);
    //     print!("img_to_ascii={:?} ", frame_start.elapsed());
    //
    //     ascii_frames.push(frame);
    //     println!("frame no.{}", ascii_frames.len());
    //     println!("total time taken frame{:?}", start.elapsed());
    // }
    //
    // for frame in &ascii_frames {
    //     // print!("\x1b[H");
    //     print!("{}", frame);
    //     thread::sleep(Duration::from_millis(100));
    // }
    //
    ffmpeg_next::init()?;

    let path = Path::new(&pathdemo);
    println!("{:?}", path);

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

    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            // let decode_time = Instant::now();
            decoder.send_packet(&packet)?;

            let mut decoded = Video::empty();

            while decoder.receive_frame(&mut decoded).is_ok() {
                // println!("decode={:?}", decode_time.elapsed());

                // let scale_time = Instant::now();
                scaler.run(&decoded, &mut scaled)?;
                // println!("scale={:?}", scale_time.elapsed());

                let mut ascii_frame = String::new();

                // let ascii = Instant::now();

                for y in 0..scaled.height() {
                    for x in 0..scaled.width() {
                        let (r, g, b) = rgb_at(&scaled, x as usize, y as usize);
                        let brightness = brightness(r, g, b);

                        let ascii_char = brightness_to_ascii(brightness);
                        // ascii_frame.push(brightness_to_ascii(brightness));
                        write!(
                            ascii_frame,
                            "\x1b[38;2;{};{};{}m{}\x1b[0m",
                            r, g, b, ascii_char
                        )
                        .unwrap();
                    }
                    ascii_frame.push('\n');
                }
                // println!("ascii={:?}", ascii.elapsed());

                // let render_time = Instant::now();
                print!("\x1b[H");
                print!("{}", ascii_frame);
                // println!("render={:?}", render_time.elapsed());
                // thread::sleep(Duration::from_millis(33));
            }
        }
    }

    Ok(())
}
