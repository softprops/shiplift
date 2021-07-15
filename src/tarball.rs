use flate2::{write::GzEncoder, Compression};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, MAIN_SEPARATOR},
};
use tar::Builder;

// todo: this is pretty involved. (re)factor this into its own crate
pub fn dir<W>(
    buf: W,
    path: &str,
) -> io::Result<()>
where
    W: Write,
{
    let mut archive = Builder::new(GzEncoder::new(buf, Compression::best()));
    fn bundle<F>(
        dir: &Path,
        f: &mut F,
        bundle_dir: bool,
    ) -> io::Result<()>
    where
        F: FnMut(&Path) -> io::Result<()>,
    {
        if fs::metadata(dir)?.is_dir() {
            if bundle_dir {
                f(dir)?;
            }
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                if fs::metadata(entry.path())?.is_dir() {
                    bundle(&entry.path(), f, true)?;
                } else {
                    f(entry.path().as_path())?;
                }
            }
        }
        Ok(())
    }

    {
        let base_path = Path::new(path).canonicalize()?;
        let mut base_path_str = match base_path.to_str() {
            Some(path) => path.to_owned(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Path uses non UTF-8 characters",
                ))
            }
        };
        if let Some(last) = base_path_str.chars().last() {
            if last != MAIN_SEPARATOR {
                base_path_str.push(MAIN_SEPARATOR)
            }
        }

        let mut append = |path: &Path| {
            let canonical = path.canonicalize()?;

            let relativized = match canonical.to_str() {
                Some(path) => path.trim_start_matches(&base_path_str[..]),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Path uses non UTF-8 characters",
                    ))
                }
            };
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
