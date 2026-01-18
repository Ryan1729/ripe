use std::path::PathBuf;

fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    let flags = xflags::parse_or_exit! {
        /// After validation, exit without packing.
        optional --no-pack
        /// Path to directory to validate and pack. Defaults to current working directory.
        optional input: PathBuf
        /// Where to place the packed output. Otherwise placed in the current working directory.
        optional -o,--output output: PathBuf
    };

    let input_dir = flags.input.unwrap_or_else(|| PathBuf::from("."));

    let mainifest_path = {
        let mut p = input_dir.clone();
        p.push(config::MANIFEST_FILENAME);
        p
    };
    
    let mainifest_string = std::fs::read_to_string(mainifest_path)?;

    let manifest = config::parse_manifest(&mainifest_string)?;

    for path in manifest.paths() {
        if let Ok(true) = std::fs::exists(path) {
            // TODO Probably need these to be relative, or copy them into the pack file and auto fix them
            // to be relative.
            // TODO? Validate stuff like that things with a given extention have the right magic numbers?
            if path == &manifest.config_path {
                let config_string = std::fs::read_to_string(path)?;

                // TODO? Anything else to validate about this? For example, are there any files this 
                // implies should also be in the pack file?
                let _config = config::parse(&config_string)?;
            }
            continue
        } else {
            return Err(format!("\"{}\" does not exist", path.display()).into());
        }
    }

    if flags.no_pack {
        println!("Skipping packing {}.pak because --no_pack was passed", manifest.name);
    } else {
        todo!("pack {manifest:?}");
    }

    Ok(())
}

fn main() {
    match inner_main() {
        Ok(()) => {},
        Err(e) => eprintln!("Error:\n{e}"),
    }
}