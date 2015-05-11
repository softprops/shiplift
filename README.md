# shiplift

[![Build Status](https://travis-ci.org/softprops/shiplift.svg)](https://travis-ci.org/softprops/shiplift)

> a rust interface for maneuvering docker containers

## docs

Find them [here](https://softprops.github.io/shiplift)

## usage

### communicating with hosts

To use shiplift you must first have a running docker daemon readily accessible. Typically this daemon
is reachable via url identified by an env named `DOCKER_HOST`. If you are using osx, boot2docker typically 
will have already set up every thing you need to get started.

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

If you are interacting with docker containers, chances are you will also need to interact with docker image information.

todoc

### containers

todoc

### misc

todoc

Doug Tangren (softprops) 2015
