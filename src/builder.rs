use self::super::Docker;
use std::io::Result;

pub struct ContainerBuilder<'a, 'b> {
  docker: &'a mut Docker,
  image: &'b str,
  hostname: Option<String>,
  user: Option<String>,
  memory: Option<u64>
}

impl<'a, 'b> ContainerBuilder<'a, 'b> {
  pub fn new(docker: &'a mut Docker, image: &'b str) -> ContainerBuilder<'a,'b> {
    ContainerBuilder {
      docker: docker, image: image,
      hostname: None, user: None,
      memory: None
    }
  }
  pub fn hostname(mut self, h: &str) -> ContainerBuilder<'a,'b> {
    self.hostname = Some(h.to_string());
    self
  }

  pub fn user(mut self, u: &str) -> ContainerBuilder<'a, 'b> {
    self.user = Some(u.to_string());
    self
  }

  pub fn build(self) -> Result<String> {
    self.docker.post("/containers/create")
  }
}
