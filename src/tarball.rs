use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{MAIN_SEPARATOR, Path};
use tar::Archive;

// todo: this is pretty involved. (re)factor this into its own crate
pub fn dir<W>(buf: W, path: &str) -> io::Result<()>
where
    W: Write,
{
    let archive = Archive::new(GzEncoder::new(buf, Compression::Best));
    fn bundle<F>(dir: &Path, f: &F, bundle_dir: bool) -> io::Result<()>
    where
        F: Fn(&Path) -> io::Result<()>,
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

    {
        let base_path = Path::new(path).canonicalize()?;
        // todo: don't unwrap
        let mut base_path_str = base_path.to_str().unwrap().to_owned();
        if let Some(last) = base_path_str.chars().last() {
            if last != MAIN_SEPARATOR {
                base_path_str.push(MAIN_SEPARATOR)
            }
        }

        let append = |path: &Path| {
            let canonical = path.canonicalize()?;
            // todo: don't unwrap
            let relativized = canonical.to_str().unwrap().trim_left_matches(
                &base_path_str[..],
            );
            if path.is_dir() {
                archive.append_dir(Path::new(relativized), &canonical)?
            } else {
                archive.append_file(
                    Path::new(relativized),
                    &mut File::open(&canonical)?,
                )?
            }
            Ok(())
        };
        bundle(Path::new(path), &append, false)?;
        archive.finish()?;
    }

    Ok(())
}
