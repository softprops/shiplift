use flate2::write::GzEncoder;
use flate2::Compression;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tar::Archive;

use errors::Result;

// todo: this is pretty involved. (re)factor this into its own crate
pub fn dir<W>(buf: W, path: &str) -> Result<()>
where
    W: Write,
{
    let archive = Archive::new(GzEncoder::new(buf, Compression::Best));
    {
        let base_path = Path::new(path).canonicalize()?;
        let mut base_path = base_path.as_path();

        if base_path.is_file() {
            // Unwrap can't return None, cause path cannot be root (`/`)
            base_path = base_path.parent().unwrap();
        }

        let mut append = |path: &Path| {
            let canonical = path.canonicalize()?;
            let relativized = canonical.strip_prefix(base_path).unwrap();

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
    } else {
        f(&dir)?;
    }
    Ok(())
}
