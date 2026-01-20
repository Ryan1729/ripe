use models::{Pak};
use platform_types::PakReader;

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
        }
    }
}

impl From<config::Error> for Error {
    fn from(e: config::Error) -> Self {
        Self::Config(e)
    }
}

impl From<ZipError> for Error {
    fn from(e: ZipError) -> Self {
        Self::Zip(e)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Self::FromUtf8(e)
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Self::TryFromInt(e)
    }
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
    
    // TODO actually read spreadsheet and get pixels out
    // TODO? Are we actually getting enough extra compression
    // out of a png to make having a png library included
    // be worth it?

    Ok(Pak {
        config,
        spritesheet: (),
    })
}
