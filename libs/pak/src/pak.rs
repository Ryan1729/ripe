use models::{Pak, Spritesheet};
use platform_types::{ARGB, PakReader};

use std::io::Read;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use std::num::TryFromIntError;
use zip::{ZipArchive, result::ZipError};



#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    Zip(ZipError),
    ZipWithPath(PathBuf, ZipError),
    FromUtf8(FromUtf8Error),
    TryFromInt(TryFromIntError),
    Png(PngError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            Config(e) => {
                write!(f, "{e}")
            },
            Zip(e) => {
                write!(f, "{e}")
            },
            ZipWithPath(path, e) => {
                write!(f, "{}: {e}", path.display())
            },
            FromUtf8(e) => {
                write!(f, "{e}")
            },
            TryFromInt(e) => {
                write!(f, "{e}")
            },
            Png(e) => {
                write!(f, "{e}")
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            Config(e) => {
                Some(e)
            },
            Zip(e) => {
                Some(e)
            },
            ZipWithPath(_, e) => {
                Some(e)
            },
            FromUtf8(e) => {
                Some(e)
            },
            TryFromInt(e) => {
                Some(e)
            },
            Png(e) => {
                Some(e)
            },
        }
    }
}

macro_rules! from_def {
    ($($from: path => $wrapper: path),+ $(,)?) => {
        $(
            impl From<$from> for Error {
                fn from(e: $from) -> Self {
                    $wrapper(e)
                }
            }
        )+
    }
}

from_def!{
    config::Error => Error::Config,
    ZipError => Error::Zip,
    FromUtf8Error => Error::FromUtf8,
    TryFromIntError => Error::TryFromInt,
    PngError => Error::Png,
}

pub const MANIFEST_FILENAME: &str = "manifest.rn";

pub fn from_reader<R>(reader: R) -> Result<Pak, Error> 
    where R: PakReader
{
    let mut archive = ZipArchive::new(reader)?;

    macro_rules! by_path {
        ($path: expr) => ({
            let path = $path;
            archive.by_path($path).map_err(|e| Error::ZipWithPath(path.clone(), e))?
        })
    }

    let manifest = {
        let mut manifest_file = by_path!(PathBuf::from(MANIFEST_FILENAME));
    
        let mut manifest_buffer = Vec::with_capacity(
            manifest_file.size().try_into()?
        );
    
        manifest_file.read_to_end(&mut manifest_buffer).map_err(ZipError::from)?;
    
        let manifest_code: String = manifest_buffer.try_into()?;
    
        config::parse_manifest(&manifest_code)?
    };

    let config = {
        let mut config_file = by_path!(&manifest.config_path);
    
        let mut config_buffer = Vec::with_capacity(
            config_file.size().try_into()?
        );
    
        config_file.read_to_end(&mut config_buffer).map_err(ZipError::from)?;
    
        let config_code: String = config_buffer.try_into()?;
    
        config::parse(&config_code)?
    };


    let mut spritesheet_file = by_path!(&manifest.spritesheet_path);

    let mut spritesheet_buffer = Vec::with_capacity(
        spritesheet_file.size().try_into()?
    );

    spritesheet_file.read_to_end(&mut spritesheet_buffer).map_err(ZipError::from)?;
    
    // TODO? Are we actually getting enough extra compression
    // out of a png to make having a png library included
    // be worth it?
    let png_frame = read_png_frame(std::io::Cursor::new(&spritesheet_buffer))?;

    let spritesheet = spritesheet_from_png_frame(&png_frame);

    Ok(Pak {
        config,
        spritesheet,
    })
}

pub type PngInfo<'info> = png::Info<'info>;

pub struct PngFrame<'info> {
    pub buffer: Vec<u8>,
    pub info: PngInfo<'info>,
}

pub type PngError = png::DecodingError;

pub fn read_png_frame<'reader>(reader: impl std::io::BufRead + std::io::Seek + 'reader) -> Result<PngFrame<'reader>, PngError> {
    let decoder = png::Decoder::new(reader);
    let mut decoding_reader = decoder.read_info()?;

    // Allocate the output buffer.
    let mut buffer = vec![0; decoding_reader.output_buffer_size().expect("Size should fit into memory")];

    // Read the next frame. Currently this function should only called once.
    let output_info = decoding_reader.next_frame(&mut buffer)?;

    let info = decoding_reader.info().clone();

    assert_eq!(output_info.width, info.width);
    assert_eq!(output_info.height, info.height);
    assert_eq!(output_info.color_type, info.color_type);
    assert_eq!(output_info.bit_depth, info.bit_depth);

    Ok(PngFrame {
        buffer,
        info,
    })
}

pub fn spritesheet_from_png_frame(frame: &PngFrame) -> Spritesheet {
    use png::ColorType::*;
    let pixel_width = frame.info.color_type.samples();

    let mut pixels = Vec::with_capacity(frame.buffer.len() / pixel_width);

    match frame.info.color_type {
        Grayscale => {
            for colour in frame.buffer.chunks(pixel_width) {
                let argb: ARGB =
                    (0xFF << 24)
                    | ((colour[0] as ARGB) << 16)
                    | ((colour[0] as ARGB) << 8)
                    | ((colour[0] as ARGB));

                pixels.push(argb);
            }
        },
        Rgb => {
            for colour in frame.buffer.chunks(pixel_width) {
                let argb: ARGB =
                    (0xFF << 24)
                    | ((colour[0] as ARGB) << 16)
                    | ((colour[1] as ARGB) << 8)
                    | ((colour[2] as ARGB));

                pixels.push(argb);
            }
        },
        Indexed => {
            // The library ensures this while decoding, so this being missing can only be
            // programmer error.
            let palette = frame.info.palette.as_ref().expect("Indexed images must have a palette!");

            for &palette_index in &frame.buffer {
                let i = usize::from(palette_index) * 3;
                
                let argb: ARGB =
                    (0xFF << 24)
                    | ((palette[i] as ARGB) << 16)
                    | ((palette[i + 1] as ARGB) << 8)
                    | ((palette[i + 2] as ARGB));

                pixels.push(argb);
            }
        },
        GrayscaleAlpha => {
            for colour in frame.buffer.chunks(pixel_width) {
                let argb: ARGB =
                    ((colour[1] as ARGB) << 24)
                    | ((colour[0] as ARGB) << 16)
                    | ((colour[0] as ARGB) << 8)
                    | ((colour[0] as ARGB));

                pixels.push(argb);
            }
        },
        Rgba => {
            for colour in frame.buffer.chunks(pixel_width) {
                let argb: ARGB =
                    ((colour[3] as ARGB) << 24)
                    | ((colour[0] as ARGB) << 16)
                    | ((colour[1] as ARGB) << 8)
                    | ((colour[2] as ARGB));

                pixels.push(argb);
            }
        },
    };

    Spritesheet {
        pixels,
        width: usize::try_from(frame.info.width).expect("Not expected to be run on less than 32 bit platforms"),
    }
}