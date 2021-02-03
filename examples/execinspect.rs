use futures::StreamExt;
use shiplift::{Docker, ExecContainerOptions};
use std::env;

#[tokio::main]
async fn main() {
    let docker = Docker::new();
    let containers = docker.containers();

    let id = env::args()
        .nth(1)
        .expect("You need to specify a container id");

    let container = containers.get(&id);

    let opts = ExecContainerOptions::builder()
        .cmd(vec!["echo", "1"])
        .attach_stdout(true)
        .attach_stderr(true)
        .build();

    let mut exec_id = String::new();
    container.exec_with_id(&opts, &mut exec_id).next().await;

    // Inspect exec
    match container.exec_inspect(&exec_id).await {
        Ok(exec_info) => {
            println!("{:#?}", exec_info);
            //exit code
            let _exit_code = exec_info.exit_code;
        }
        Err(e) => eprintln!("{:?}", e),
    }
}
