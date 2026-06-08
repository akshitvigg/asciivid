use image::{DynamicImage, GenericImageView, ImageError, imageops::FilterType::Lanczos3};
use std::env;

fn get_img_path() -> Result<String, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("add image path");
        return Err("add image path".to_string());
    }

    let image_name = &args[1];
    Ok(image_name.to_string())
}

fn load_image(path: &str) -> Result<DynamicImage, ImageError> {
    image::ImageReader::open(path)?.decode()
}

fn resize_image(img: DynamicImage) -> DynamicImage {
    image::DynamicImage::resize(&img, 150, 75, Lanczos3)
}

const RAMP: &str = " .:-=+*#%@";
const MAGIC_NUM: f64 = (RAMP.len() - 1) as f64 / 255.0;

fn brightness_to_ascii(pixel: u8) -> char {
    let ramp_ind = (pixel) as f64 * MAGIC_NUM;
    let ramp_abs = ramp_ind.round();
    let ascii_char = RAMP.as_bytes()[ramp_abs as usize] as char;
    return ascii_char;
}

fn render_ascii(img: DynamicImage) {
    let mut prev_y = 0;
    let gray = img.grayscale();

    for (_, y, pixel) in gray.pixels() {
        if prev_y != y {
            print!("\n");
            print!("{}", brightness_to_ascii(pixel[0]));
        } else {
            print!("{}", brightness_to_ascii(pixel[0]));
        }

        prev_y = y;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = get_img_path()?;
    let img = load_image(&path)?;
    let resized_img = resize_image(img);
    render_ascii(resized_img);

    Ok(())
}
