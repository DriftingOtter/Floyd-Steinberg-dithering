use std::env;
use image::ImageReader;

fn main() {
    let args = get_command_line_args()
        .expect("Failed to take in command line arguments.");

    // read the image data into buffer as raw samples
    let img_buffer = ImageReader::open(&args.0)
        .unwrap().decode().unwrap();

    let width = img_buffer.width();
    let height = img_buffer.height();

    // take out only the luminance
    let samples: Vec<u8> = img_buffer
        .to_luma8().as_flat_samples().samples.to_vec();

    let dithered_samples: Vec<u8> = apply_dither(samples.clone(), 2, width, height)
        .expect("Failed to dither image sample array");

    // create new image buffer
    let mut new_img = image::GrayImage::new(width, height);
    
    // Copy data into the image buffer
    new_img.pixels_mut().enumerate().for_each(|(sample, px)| {
        px[0] = dithered_samples[sample];
    });

    new_img.save(&args.1).expect("Failed to save dithered image");
}

fn get_command_line_args() -> Result<(String, String), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Incorrect amount of command-line arguments, dthr <input_file_name> <output_file_name>");
        return Err(());
    }

    return Ok((args[1].clone(), args[2].clone()));
}

fn re_quantize(sample: f32, shade_count: u8) -> f32 {
    return ((sample * (shade_count as f32 - 1.0)).round()) / (shade_count as f32 - 1.0);
}

fn apply_dither(image: Vec<u8>, shade_count: u8, width: u32, height: u32) -> Result<Vec<u8>, ()> {

    // normalize the array to floating point values for precise arithmetic
    let mut normalized_array: Vec<f32> = image.iter()
        .map(|&sample| sample as f32 / 255.0)
        .collect();

    if height == 0 || width == 0 {
        return Err(());
    }

    for row in 0..height {
        for col in 0..width {
            let index = (row * width + col) as usize;
            let original_sample = normalized_array[index];
            let new_sample = re_quantize(original_sample, shade_count);

            normalized_array[index] = new_sample;

            let error = original_sample - new_sample;

            // Propagate error to the right pixel
            if col < width - 1 {
                let right_index = (row * width + (col + 1)) as usize;
                normalized_array[right_index] += error * 7.0 / 16.0;
            }

            // Propagate error to the bottom-left pixel
            if row < height - 1 {
                if col > 0 {
                    let bottom_left_index = ((row + 1) * width + (col - 1)) as usize;
                    normalized_array[bottom_left_index] += error * 3.0 / 16.0;
                }

                // Propagate error to the bottom pixel
                let bottom_index = ((row + 1) * width + col) as usize;
                normalized_array[bottom_index] += error * 5.0 / 16.0;

                // Propagate error to the bottom-right pixel
                if col < width - 1 {
                    let bottom_right_index = ((row + 1) * width + (col + 1)) as usize;
                    normalized_array[bottom_right_index] += error / 16.0;
                }
            }
        }
    }

    let reconstructed_array: Vec<u8> = normalized_array.iter()
        .map(|&sample| (sample * 255.0).clamp(0.0, 255.0) as u8)
        .collect();

    return Ok(reconstructed_array);
}

