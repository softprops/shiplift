use futures::Future;
use shiplift::Docker;

fn main() {
    let docker = Docker::host("http://yourhost".parse().unwrap());

    let fut = docker
        .ping()
        .map(|pong| println!("Ping: {}", pong))
        .map_err(|e| eprintln!("Error: {}", e));

    tokio::run(fut);
}
