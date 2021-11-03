use std::fs;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use clap::{App, Arg, ArgMatches};
use walkdir::WalkDir;

fn arguments() -> ArgMatches<'static> {
    let matches = App::new("br")
        .version("0.1")
        .about("Batch rename, renames all files in the specified folders to their blake3 hashsum")
        .arg(
            Arg::with_name("path")
                .long("path")
                .value_name("PATH")
                .help(
                    "Path to directory for files to rename. Defaults to current working directory",
                )
                .multiple(true)
                .required(true),
        )
        .get_matches();

    matches
}

fn hash(path: &Path) -> Result<blake3::Hash> {
    let mut hasher = blake3::Hasher::new();
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);

    std::io::copy(&mut reader, &mut hasher)?;
    Ok(hasher.finalize())
}

fn rename<P: AsRef<Path> + ?Sized, Q: AsRef<Path>>(old: &P, new: &Q) -> Result<()> {
    fs::rename(old, new)?;
    Ok(())
}

fn main() -> Result<()> {
    let matches = arguments();
    /* Default to current working directory */
    let path = matches.value_of("path").unwrap_or(".");
    let path = fs::canonicalize(path)?;
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir())
    {
        // Save the extension, it will get overwritten when
        // setting the new filname
        let extension = entry.path().extension();

        let hash = hash(entry.path())?.to_hex();

        let mut new_path = entry.path().with_file_name(&hash.as_str());
        if let Some(ext) = extension {
            new_path.set_extension(&ext);
        }

        rename(entry.path(), &new_path)?;
    }
    Ok(())
}
