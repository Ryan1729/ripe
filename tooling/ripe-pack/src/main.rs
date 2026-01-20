use std::path::PathBuf;

fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let flags = xflags::parse_or_exit! {
        /// After validation, exit without packing.
        optional --no-pack
        /// Path to directory to validate and pack. Defaults to current working directory.
        optional input: PathBuf
        /// Path to directory to place the packed output into. Otherwise placed in the current working directory.
        optional -o,--output output: PathBuf
    };

    let input_dir = flags.input.unwrap_or_else(|| PathBuf::from("."));

    let manifest_path = input_dir.join(pak::MANIFEST_FILENAME);
    
    let manifest_string = std::fs::read_to_string(manifest_path)?;

    let manifest = config::parse_manifest(&manifest_string)?;

    macro_rules! fatal {
        ($($arg:tt)*) => {
            return Err(format!($($arg)*).into())
        }
    }

    for rel_path in manifest.paths() {
        if rel_path.is_absolute() {
            fatal!("Manifest paths must be relative. {} is not.", rel_path.display())
        }
        let path = input_dir.join(rel_path);

        if let Ok(true) = std::fs::exists(&path) {
            // TODO? Validate stuff like that things with a given extention have the right magic numbers?
            if path == manifest.config_path {
                let config_string = std::fs::read_to_string(path)?;

                // TODO? Anything else to validate about this? For example, are there any files this 
                // implies should also be in the pack file?
                let _config = config::parse(&config_string)?;
            }
            continue
        } else {
            fatal!("\"{}\" does not exist", path.display());
        }
    }

    if flags.no_pack {
        println!("Skipping packing {}.pak because --no_pack was passed", manifest.name);
    } else {
        use std::io::Write;

        let output_dir = flags.output.unwrap_or_else(|| PathBuf::from("."));

        let output_path = output_dir.join(format!("{}.pak", manifest.name));

        let output_file = std::fs::File::create(&output_path)?;

        let mut zip = zip::ZipWriter::new(output_file);

        for rel_path in manifest.paths() {
            if rel_path.is_absolute() {
                fatal!("Manifest paths must be relative. {} is not.", rel_path.display())
            }
            let path = input_dir.join(rel_path);

            zip.start_file_from_path(
                &rel_path,
                zip::write::SimpleFileOptions::default(),
            )?;

            zip.write_all(&std::fs::read(path)?)?;
        }

        zip.start_file_from_path(
            pak::MANIFEST_FILENAME,
            zip::write::SimpleFileOptions::default(),
        )?;

        zip.write_all(manifest_string.as_bytes())?;

        zip.finish()?;

        println!("Wrote {}", output_path.display());
    }

    Ok(())
}

use std::process::ExitCode;

fn main() -> ExitCode {
    match inner_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error:\n{e}");
            ExitCode::FAILURE
        },
    }
}