
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::path::{Path, MAIN_SEPARATOR};
use std::io::{self, Write};
use tar::Archive;

// todo: this is pretty involved. (re)factor this into its own crate
pub fn dir<W>(buf: W, path: &str) -> io::Result<()>
    where W: Write
{
    let archive = Archive::new(GzEncoder::new(buf, Compression::Best));
    fn bundle<F>(dir: &Path, f: &F, bundle_dir: bool) -> io::Result<()>
        where F: Fn(&Path) -> io::Result<()>
    {
        if try!(fs::metadata(dir)).is_dir() {
            if bundle_dir {
                try!(f(&dir));
            }
            for entry in try!(fs::read_dir(dir)) {
                let entry = try!(entry);
                if try!(fs::metadata(entry.path())).is_dir() {
                    try!(bundle(&entry.path(), f, true));
                } else {
                    try!(f(&entry.path().as_path()));
                }
            }
        }
        Ok(())
    }

    {
        let base_path = try!(Path::new(path).canonicalize());
        // todo: don't unwrap
        let mut base_path_str = base_path.to_str().unwrap().to_owned();
        if let Some(last) = base_path_str.chars().last() {
            if last != MAIN_SEPARATOR {
                base_path_str.push(MAIN_SEPARATOR)
            }
        }

        let append = |path: &Path| {
            let canonical = try!(path.canonicalize());
            // todo: don't unwrap
            let relativized = canonical.to_str().unwrap().trim_left_matches(&base_path_str[..]);
            if path.is_dir() {
                try!(archive.append_dir(Path::new(relativized), &canonical))
            } else {
                try!(archive.append_file(Path::new(relativized), &mut try!(File::open(&canonical))))
            }
            Ok(())
        };
        try!(bundle(Path::new(path), &append, false));
        try!(archive.finish());
    }

    Ok(())
}
