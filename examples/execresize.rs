use shiplift::{Docker, Exec, ExecContainerOptions, ExecResizeOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let mut args = env::args().skip(1);

    // First argument is container id
    let id = args.next().expect("You need to specify a container id");
    // Second is width
    let width: u64 = args.next().map_or(Ok(0), |s| s.parse::<u64>()).unwrap();
    // Third is height
    let height: u64 = args.next().map_or(Ok(0), |s| s.parse::<u64>()).unwrap();

    // Create an exec instance
    let exec_opts = ExecContainerOptions::builder()
        .cmd(vec!["echo", "123"])
        .attach_stdout(true)
        .attach_stderr(true)
        .build();
    let exec = Exec::create(&docker, &id, &exec_opts).await.unwrap();

    // Resize its window with given parameters
    let resize_opts = ExecResizeOptions::builder()
        .width(width)
        .height(height)
        .build();
    exec.resize(&resize_opts).await.unwrap();
}
