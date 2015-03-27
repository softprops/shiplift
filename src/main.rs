extern crate shiplift;

use shiplift::Docker;

fn main() {
  let mut docker = Docker::new();
  match docker.images().search("rust") {
    Ok(e) => println!("-> {:?}", e),
    Err(e) => panic!("<- {:?}", e)
  };
}
