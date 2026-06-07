use image::{GenericImageView, ImageReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let img = ImageReader::open("toji.jpg")?.decode()?;

    // println!("width: {}", img.width());
    // println!("height: {}", img.height());

    let gray = img.grayscale();

    let ramp = " .:-=+*#%@";

    let mut prev_y = 0;

    for (x, y, pixel) in gray.pixels() {
        // println!("{}, {} {},  {:?}", i, x, y, pixel[0]);
        let ramp_ind = pixel[0] as f64 * 0.0353;
        let rampabs = ramp_ind.round();

        let ascichar = ramp.as_bytes()[rampabs as usize] as char;
        if prev_y != y {
            print!("\n");
            print!("{}", ascichar);
        } else {
            print!("{}", ascichar);
        }

        prev_y = y;

        // in this loop if ramp_ind = pixel[i] *0.0353
        // and print ramp[ramp_ind] sort of thig ?
    }
    // println!("{}", ramp.len());

    Ok(())
}
