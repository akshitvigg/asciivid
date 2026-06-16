use crossterm::terminal::size;
use image::{DynamicImage, GenericImageView, ImageError, imageops::FilterType::Lanczos3};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use terminal_size::terminal_size;

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
    image::DynamicImage::resize_exact(&img, w as u32, h as u32, Lanczos3)
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
    // let path = get_img_path()?;

    let mut frames: Vec<PathBuf> = fs::read_dir("/home/akira/marchFrames/")?
        .map(|e| e.unwrap().path())
        .collect();

    frames.sort();

    for frame in frames {
        let start = Instant::now();
        let img = load_image(&frame)?;
        let resized_img = resize_image(img, w, h);
        render_ascii(&resized_img);
        print!("{:?}", start.elapsed());
        // thread::sleep(Duration::from_millis(33));
        // print!("\x1b[2J");
        // print!("\x1b[H");
    }

    Ok(())
}
