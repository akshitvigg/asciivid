use image::{GenericImageView, ImageReader, imageops::FilterType::Gaussian};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("toji.jpg")?.decode()?;

    print!("{:?}", img.dimensions());

    img = image::DynamicImage::resize(&img, 150, 75, Gaussian);

    let gray = img.grayscale();

    let ramp = " .:-=+*#%@";

    let mut prev_y = 0;
    let magic_num = (ramp.len() - 1) as f64 / 255.0;

    for (x, y, pixel) in gray.pixels() {
        let ramp_ind = pixel[0] as f64 * magic_num;
        let ramp_abs = ramp_ind.round();
        let asciichar = ramp.as_bytes()[ramp_abs as usize] as char;

        if prev_y != y {
            print!("\n");
            print!("{}", asciichar);
        } else {
            print!("{}", asciichar);
        }

        prev_y = y;
    }

    Ok(())
}
