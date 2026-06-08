use image::{GenericImageView, ImageReader, imageops::FilterType::Gaussian};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("toji.jpg")?.decode()?;

    img = image::DynamicImage::resize(&img, 100, 100, Gaussian);
    print!("{:?}", img.dimensions());

    let gray = img.grayscale();

    let ramp = " .:-=+*#%@";
    // let ramp = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    // let ramp = " .'`^\",:;Il!i~+_-?][}{1)(|\\/*tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

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
