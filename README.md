# shiplift

[![Build Status](https://travis-ci.org/softprops/shiplift.svg)](https://travis-ci.org/softprops/shiplift) [![crates.io](http://meritbadge.herokuapp.com/shiplift)](https://crates.io/crates/shiplift) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

> a rust interface for maneuvering [docker](https://www.docker.com/) containers

## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
shiplift = "0.3"
```

## docs

Find them [here](https://softprops.github.io/shiplift).

## usage

Some small example programs can be found in this repository's [examples directory](https://github.com/softprops/shiplift/tree/master/examples).

### communicating with hosts

To use shiplift, you must first have a docker daemon readily accessible. Typically this daemon processs
is resolvable via a url specified by an env var named `DOCKER_HOST`. If you are using osx, [docker-machine](https://docs.docker.com/machine/) typically
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

#### creating new image by pulling an existing image

```rust
use shiplift::PullOptions;
let output = images.pull(
  &PullOptions::builder().image("redis:2.8.18").build()
).unwrap();
for o in output {
  println!("{:?}", o);
}
```

### build an image from the contents of a directory containing a Dockerfile

the following is equivalent to `docker build -t shiplift_test .`

```rust
use shiplift::BuildOptions;

let output = images.build(
     &BuildOptions::builder(".").tag("shiplift_test").build()
).unwrap();
for o in output {
    println!("{:?}", o);
}
```

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

## roadmap

There are plans on switching from rustc-serialize to serde for serialization in 0.4.0 this should not have
major impact on current interfaces.

Doug Tangren (softprops) 2015-2016
