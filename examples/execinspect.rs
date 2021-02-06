use futures::StreamExt;
use shiplift::{Docker, Exec, ExecContainerOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let mut args = env::args().skip(1);

    // First argument is container id
    let id = args.next().expect("You need to specify a container id");
    // Rest is command to run in the container
    let cmd = args.collect::<Vec<String>>();
    println!("{} {:?}", id, cmd);

    // Create options with specified command
    let opts = ExecContainerOptions::builder()
        .cmd(cmd.iter().map(String::as_str).collect())
        .attach_stdout(true)
        .attach_stderr(true)
        .build();

    let exec = Exec::create(&docker, &id, &opts).await.unwrap();

    println!("{:#?}", exec.inspect().await.unwrap());

    let mut stream = exec.start();

    stream.next().await;

    println!("{:#?}", exec.inspect().await.unwrap());
}
