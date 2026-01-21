//Read in the png and:
//     * output the data as a text array
//     * output a transformed copy of the png

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};

#[cfg(true)]
mod filenames {
    pub const IMAGE_FILENAME: &'static str = "../../assets/gfx.png";
    pub const INLINE_OUTPUT_FILENAME: &'static str = "../../libs/assets/src/gfx.in";
    pub const TRANSFORMED_OUTPUT_FILENAME: &'static str = "../../examples/gfx2h.png";
}

// for testing
#[cfg(false)]
mod filenames {
    pub const IMAGE_FILENAME: &'static str = "assets/pallete.png";
    pub const INLINE_OUTPUT_FILENAME: &'static str = "out.txt";
    pub const TRANSFORMED_OUTPUT_FILENAME: &'static str = "gfx2h.png";
}

use filenames::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frame = pak::read_png_frame(BufReader::new(File::open(IMAGE_FILENAME)?));
    
    println!(
        "{} : {:?}",
        IMAGE_FILENAME,
        (
            frame.info.width,
            frame.info.height,
            frame.info.color_type,
            frame.info.bit_depth,
        )
    );

    //
    // Write into the inline output
    //
    {
        let inline_output_filename = INLINE_OUTPUT_FILENAME;
    
        let mut file = File::create(inline_output_filename)?;

        let spritesheet = pak::spritesheet_from_png_frame(&png_frame);
    
        assert_eq!(spritesheet.width, gfx_sizes::GFX_WIDTH, "Input PNG was not the right width");
        assert_eq!(spritesheet.pixels.len(), gfx_sizes::GFX_LENGTH, "Input PNG was not the right length");

        let mut output = String::with_capacity(
            spritesheet.pixels.len() * "0xFFFFFFFF, ".len()
            // Newlines for each row
            + 1024
            // Extra for start and end of array
            + 8
        );
        output.push_str("[\n");
        for chunk in spritesheet.pixels.chunks(gfx_sizes::GFX_WIDTH) {
            for colour in chunk.iter() {
                output.push_str(&format!("0x{colour:08X}, "));
            }
            output.push('\n');
        }
        output.push_str("]\n");
    
        file.write_all(output.as_bytes())?;
    
        println!("overwrote {}", inline_output_filename);
    }
    //
    // Copy input at double height to new location
    //
    {
        let transformed_output_filename = TRANSFORMED_OUTPUT_FILENAME;
    
        let file = File::create(transformed_output_filename)?;

        let ref mut writer = BufWriter::new(file);
        
        let mut new_info = frame.info.clone();

        new_info.height *= 2;

        fn to_usize(n: u32) -> usize { usize::try_from(n).expect("Not expected to be run on less than 32 bit platforms") }

        let new_height = to_usize(new_info.height);
        let new_width = to_usize(new_info.width);

        let old_height = to_usize(info.height);
        let old_width = to_usize(info.width);

        assert_eq!(new_width, old_width);

        let bpp = new_info.bytes_per_pixel();

        let encoder = png::Encoder::with_info(writer, new_info)?;

        let mut writer = encoder.write_header()?;

        let expected_length = new_height * new_width * bpp;

        let mut data = Vec::with_capacity(expected_length);

        for y_index in 0..old_height {
            for x_index in 0..old_width {
                for i in 0..bpp {
                    data.push(frame.buffer[y_index * old_width * bpp + x_index * bpp + i])
                }
            }

            // Add extra row
            for x_index in 0..old_width {
                for i in 0..bpp {
                    data.push(frame.buffer[y_index * old_width * bpp + x_index * bpp + i])
                }
            }
        }
        
        assert_eq!(data.len(), expected_length);

        writer.write_image_data(&data)?;
    
        println!("overwrote {}", transformed_output_filename);
    }

    Ok(())
}
