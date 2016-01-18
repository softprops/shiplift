
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::path::{Path, MAIN_SEPARATOR};
use std::io::{self, Write, Read};
use tar::Archive;

// todo: this is pretty involved. factor this into its own crate
pub fn dir<W>(buf: W, path: &str) -> io::Result<()> where W: Write {
    let archive = Archive::new(GzEncoder::new(buf, Compression::Best));
    fn bundle(dir: &Path, cb: &Fn(&Path), bundle_dir: bool) -> io::Result<()> {
        if try!(fs::metadata(dir)).is_dir() {
            if bundle_dir {
                cb(&dir);
            }
            for entry in try!(fs::read_dir(dir)) {
                let entry = try!(entry);
                if try!(fs::metadata(entry.path())).is_dir() {
                    try!(bundle(&entry.path(), cb, true));
                } else {
                    cb(&entry.path().as_path());
                }
            }
        }
        Ok(())
    }

    {
        let base_path = Path::new(path).canonicalize().unwrap();
        let mut base_path_str = base_path.to_str().unwrap().to_owned();
        if base_path_str.chars().last().unwrap() != MAIN_SEPARATOR {
            base_path_str.push(MAIN_SEPARATOR)
        }

        let append = |path: &Path| {
            let canonical = path.canonicalize().unwrap();
            let relativized = canonical.to_str().unwrap().trim_left_matches(&base_path_str[..]);
            if path.is_dir() {
                archive.append_dir(Path::new(relativized), &canonical).unwrap();
            } else {
                archive.append_file(Path::new(relativized), &mut File::open(&canonical).unwrap()).unwrap();
            }
        };
        try!(bundle(Path::new(path), &append, false));
        try!(archive.finish());
    }

    Ok(())
}
