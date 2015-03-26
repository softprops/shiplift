extern crate shiplift;

use shiplift::Docker;

fn main() {
   match Docker::new().info() {
       Ok(e) => println!("-> {:?}", e),
       Err(e) => panic!("<- {:?}", e)
   };
}
