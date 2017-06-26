extern crate shiplift;

use shiplift::Docker;
use std::env;

fn main() {
    let docker = Docker::new();
    if let Some(id) = env::args().nth(1) {
        let mut logs = docker
            .containers()
            .get(&id)
            .logs(&Default::default())
            .unwrap();
        std::io::copy(&mut logs, &mut std::io::stdout()).unwrap();
    }
}
