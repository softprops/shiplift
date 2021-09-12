use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, MAIN_SEPARATOR},
};
use tar::Builder;

use crate::errors::Error;
use crate::errors::Result;

// todo: this is pretty involved. (re)factor this into its own crate
pub fn dir<W>(
    buf: W,
    path: &str,
) -> Result<()>
where
    W: Write,
{
    let mut archive = Builder::new(GzEncoder::new(buf, Compression::best()));

    {
        let base_path = Path::new(path).canonicalize()?;

        let mut base_path_str = base_path.to_str().ok_or_else(|| Error::PathNotUtf8(base_path.to_path_buf()))?.to_owned();
        if let Some(last) = base_path_str.chars().last() {
            if last != MAIN_SEPARATOR {
                base_path_str.push(MAIN_SEPARATOR)
            }
        }

        let mut append = |path: &Path| {
            let canonical = path.canonicalize()?;
            let relativized = canonical
                .to_str()
                .ok_or_else(|| Error::PathNotUtf8(canonical.to_path_buf()))?
                .trim_start_matches(&base_path_str[..]);
            if path.is_dir() {
                archive.append_dir(Path::new(relativized), &canonical)?
            } else {
                archive.append_file(Path::new(relativized), &mut File::open(&canonical)?)?
            }
            Ok(())
        };
        bundle(Path::new(path), &mut append, false)?;
    }
    archive.finish()?;

    Ok(())
}

fn bundle<F>(
    dir: &Path,
    f: &mut F,
    bundle_dir: bool,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
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
