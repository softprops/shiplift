# shiplift

[![Build Status](https://travis-ci.org/softprops/shiplift.svg)](https://travis-ci.org/softprops/shiplift)

> a rust interface for maneuvering docker containers

## docs

Find them [here](https://softprops.github.io/shiplift)

## usage

### communicating with hosts

To use shiplift you must first have a running docker daemon readily accessible. Typically this daemon
is reachable via url identified by an env named `DOCKER_HOST`. If you are using osx, [docker-machine](https://docs.docker.com/machine/) typically
will have already set up every thing you need to get started when you run `docker-machine env {envid}`.

```rust
extern crate shiplift;
let docker = shiplift::Docker::new();
```

If you wish to be more explicit you can provide a host in the form of a `url.Url`.

```rust
extern crate shiplift;
extern crate url;

use shiplift::Docker;
use url::Url;

let docker = Docker::host(Url::parse("http://yourhost").unwrap());
```

### images

If you are interacting with docker containers, chances are you will also need to interact with docker image information. You can interact docker images with `docker.images()`.

```rust
extern crate shiplift;

use shiplift::Docker;

let docker = Docker.new();
let images = docker.images();
```

#### list host-local images

```rust
for i in images.list(&Default::default()).unwrap() {
  println!("-> {:?}", i);
}
```

#### find remote images

```rust
for i in image.search("rust").unwrap() {
  println!("- {:?}", i);
}
```

#### creating new images from existing image

todo

#### accessing image info

```rust
let img = images.get("imagename");
```

##### inspecting image info

```rust
println!("- {:?}", img.inspect().unwrap());
```

##### getting image history

```rust
for h in img.history().unwrap() {
  println!("- {:?}", h);
}
```

###### deleting image

```rust
println!("- {:?}", img.delete().unwrap());
```

### containers

Containers are instances of images. To gain access to this interface use `docker.containers()`

```rust
extern crate shiplift;

use shiplift::Docker;

let docker = Docker.new();
let containers = docker.containers();
```

#### listing host local containers

```rust
for c in containers.list(&Default::default()).unwrap() {
  println!("- {:?}", c);
}
```

#### get a container reference

```rust
let container = containers.get("containerid");
```

#### inspect container details

```rust
println!("- {:?}", container.inspect());
```

#### access `top` info

```rust
println!("- {:?}", container.top().unwrap());
```

#### view container logs

(todoc)

#### view a list of container changes

```rust
for c in container.changes().unwrap() {
  println!("- {:?}", c);
}
```

#### stream container stats

```rust
for stats in container.stats().unwrap() {
  println!("- {:?}", stats);
}
```

### stop, start, restart container

```rust
container.stop();
container.start();
container.restart();
```

### misc

todoc

Doug Tangren (softprops) 2015
