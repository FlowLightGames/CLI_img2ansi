use anyhow::Result;
use clap::Parser;
use image::{ImageBuffer, Rgba};
use std::env;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Simple CLI example
#[derive(Parser, Debug)]
struct Args {
    /// Input file
    #[arg(short, long)]
    input_image: String,

    /// Output path
    #[arg(short, long, default_value_os = "./")]
    output_path: PathBuf,

    /// "HIGH RES" config option; Template that ignores other options
    #[arg(long, default_value_t = false)]
    high_res: bool,

    /// Custom ascii that represets a pixel
    #[arg(short, long, default_value_t = String::from("██"))]
    pixel_ascii: String,
}

fn rgb_to_ansi(color: &[u8; 4]) -> String {
    format!("\x1b[38;2;{};{};{}m", color[0], color[1], color[2])
}

fn rgb_to_bg_ansi(color: &[u8; 4]) -> String {
    format!("\x1b[48;2;{};{};{}m", color[0], color[1], color[2])
}

fn pixel_slice_to_ansi(
    length: u64,
    color: &Rgba<u8>,
    pixel_ascii: &str,
    empty_pixel_string: &str,
) -> String {
    let mut output: String = "".to_string();

    if length > 0 {
        if color.0[3] != 255 {
            output.push_str(&get_empty_slice(length, empty_pixel_string))
        } else {
            output.push_str(&rgb_to_ansi(&color.0));
            for _n in 0..length {
                output.push_str(pixel_ascii);
            }
            output.push_str("\x1b[0m");
        }
    }
    output
}

fn get_empty_slice(length: u64, empty_pixel_string: &str) -> String {
    let mut output: String = "".to_string();
    for _n in 0..length {
        output.push_str(empty_pixel_string);
    }
    output
}

fn is_valid_position(test: (u32, u32), dimensions: (u32, u32)) -> bool {
    if (test.0 < dimensions.0) && (test.1 < dimensions.1) {
        return true;
    }
    false
}
// TODO fix when bottom pixel is color and top is empty
fn get_high_res_ascii(
    position: (u32, u32),
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    dimensions: (u32, u32),
) -> String {
    let mut output = "".to_string();

    let mut top_rgba: Option<[u8; 4]> = None;
    let mut bottom_rgba: Option<[u8; 4]> = None;

    if is_valid_position(position, dimensions) {
        top_rgba = Some(img.get_pixel(position.0, position.1).0);
    }

    let bottom_position = (position.0, position.1 + 1);

    if is_valid_position(bottom_position, dimensions) {
        bottom_rgba = Some(img.get_pixel(bottom_position.0, bottom_position.1).0);
    }

    if let Some(color) = top_rgba {
        if color[3] != 255 {
            top_rgba = None
        }
    }

    if let Some(color) = bottom_rgba {
        if color[3] != 255 {
            bottom_rgba = None
        }
    }

    if top_rgba.is_some() {
        output.push_str(&rgb_to_ansi(&top_rgba.unwrap()));

        if bottom_rgba.is_some() {
            output.push_str(&rgb_to_bg_ansi(&bottom_rgba.unwrap()));
        }
        output.push('▀');
        output.push_str("\x1b[0m");
    } else {
        if bottom_rgba.is_some() {
            output.push_str(&rgb_to_ansi(&bottom_rgba.unwrap()));
            output.push('▄');
            output.push_str("\x1b[0m");
        } else {
            output.push_str(" ");
        }
    }
    output
}

fn main() -> Result<()> {
    // Get image path from command line
    let mut args = Args::parse();

    let mut empty_pixel_string = "".to_string();
    for _n in 0..args.pixel_ascii.chars().count() {
        empty_pixel_string.push(' ');
    }

    let img = image::open(args.input_image).expect("Expected to open image");
    let img = img.to_rgba8();

    let (width, height) = img.dimensions();

    let mut output: String = "".to_string();

    if args.high_res {
        for y in 0..(height / 2) {
            let mut x = 0;
            while x < width {
                output.push_str(&get_high_res_ascii((x, y * 2), &img, (width, height)));
                x += 1;
            }
            output.push('\n');
        }
    } else {
        for y in 0..height {
            let mut x = 0;
            let mut current_color = img.get_pixel(x, y);
            let mut current_slice_len: u64 = 0;
            while x < width {
                let tmp_color = img.get_pixel(x, y);
                if tmp_color == current_color {
                    current_slice_len += 1
                } else {
                    output.push_str(&pixel_slice_to_ansi(
                        current_slice_len,
                        current_color,
                        &args.pixel_ascii,
                        &empty_pixel_string,
                    ));
                    current_color = tmp_color;
                    current_slice_len = 1;
                }
                x += 1;
            }
            output.push_str(&pixel_slice_to_ansi(
                current_slice_len,
                current_color,
                &args.pixel_ascii,
                &empty_pixel_string,
            ));
            output.push('\n');
        }
    }

    print!("{output}");

    args.output_path.push("output.ansi");

    let mut file = File::create(args.output_path)?;
    writeln!(file, "{output}")?;
    Ok(())
}
