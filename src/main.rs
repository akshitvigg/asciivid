use crossterm::terminal::size;
use ffmpeg_next::codec::{Context, context};
use ffmpeg_next::format::{Pixel, input};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::Flags;
use image::imageops::FilterType::Nearest;
use image::{DynamicImage, GenericImageView, ImageError};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (w, h) = size()?;
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
    //
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
    //
    // let mut scaler = Context::get(
    //     decoder.format(),
    //     decoder.width(),
    //     decoder.height(),
    //     Pixel::RGB24,
    //     decoder.width(),
    //     decoder.height(),
    //     Flags::BILINEAR,
    // )?;

    println!("width={}, height={}", decoder.width(), decoder.height());
    println!("format={:?}", decoder.format());

    for (i, (stream, packet)) in ictx.packets().enumerate() {
        println!("packet #{} {:?}", i, stream);
    }

    Ok(())
}
