use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{MAIN_SEPARATOR, Path};
use tar::Archive;

use errors::Result;
use errors::Error;
use errors::ErrorKind as EK;

// todo: this is pretty involved. (re)factor this into its own crate
pub fn dir<W>(buf: W, path: &str) -> Result<()>
    where
        W: Write,
{
    let archive = Archive::new(GzEncoder::new(buf, Compression::Best));

    {
        let base_path           = Path::new(path).canonicalize()?;
        let mut base_path_str   = base_path
            .to_str()
            .ok_or_else(|| EK::Utf8)
            .map_err(Error::from_kind)?
            .to_owned();

        if let Some(last) = base_path_str.chars().last() {
            if last != MAIN_SEPARATOR {
                base_path_str.push(MAIN_SEPARATOR)
            }
        }

        let append = |path: &Path| {
            let canonical = path.canonicalize()?;
            let relativized = canonical
                .to_str()
                .ok_or_else(|| EK::Utf8)
                .map_err(Error::from_kind)?
                .trim_left_matches(&base_path_str[..]);

            if path.is_dir() {
                archive.append_dir(Path::new(relativized), &canonical)?
            } else {
                archive.append_file(Path::new(relativized), &mut File::open(&canonical)?)?
            }
            Ok(())
        };
        bundle(Path::new(path), &append, false)?;
        archive.finish()?;
    }

    Ok(())
}

fn bundle<F>(dir: &Path, f: &F, bundle_dir: bool) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
{
    if fs::metadata(dir)?.is_dir() {
        if bundle_dir {
            f(&dir)?;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if fs::metadata(entry.path())?.is_dir() {
                bundle(&entry.path(), f, true)?;
            } else {
                f(&entry.path().as_path())?;
            }
        }
    }
    Ok(())
}