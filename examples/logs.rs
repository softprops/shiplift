extern crate shiplift;

use shiplift::{Docker, LogsOptions};
use std::env;

fn main() {
    let docker = Docker::new(None).unwrap();
    if let Some(id) = env::args().nth(1) {
        let mut logs = docker
            .containers()
            .get(&id)
            .logs(&LogsOptions::builder().stdout(true).build())
            .unwrap();
        std::io::copy(&mut logs, &mut std::io::stdout()).unwrap();
    }
}
