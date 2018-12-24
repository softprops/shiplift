# shiplift

[![Build Status](https://travis-ci.org/softprops/shiplift.svg)](https://travis-ci.org/softprops/shiplift) [![crates.io](http://meritbadge.herokuapp.com/shiplift)](https://crates.io/crates/shiplift) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE) [![Released API docs](https://docs.rs/shiplift/badge.svg)](http://docs.rs/shiplift) [![Master API docs](https://img.shields.io/badge/docs-master-green.svg)](https://softprops.github.io/shiplift)

> a rust interface for maneuvering [docker](https://www.docker.com/) containers

## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
shiplift = "0.4"
```

## usage

### communicating with hosts

To use shiplift, you must first have a docker daemon readily accessible. Typically this daemon processs
is resolvable via a url specified by an env var named `DOCKER_HOST`.

```rust
let docker = shiplift::Docker::new();
```

If you wish to be more explicit you can provide a host in the form of a `url.Url`.

```rust
use shiplift::Docker;
use url::Url;

let docker = Docker::host(Url::parse("http://yourhost").unwrap());
```

### Examples

Many small runnable example programs can be found in this repository's [examples directory](https://github.com/softprops/shiplift/tree/master/examples).

## planned changes

* give image pull chunked json a proper type

Doug Tangren (softprops) 2015-2018
