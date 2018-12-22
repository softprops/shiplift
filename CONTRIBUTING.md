# Contributing to shiplift
Contributing to shiplift isn't limited to just filing bugs, users are more than welcomed to make suggestions, report any issue they may find, and make pull requests to help make shiplift better.

## Working on shiplift
### Prerequisites
* The [rust toolchain](https://rustup.rs/)
* [Git](https://git-scm.com/)


### Getting shiplift
1. Fork a copy of our repo
2. Open up Git in an environment of your choice
3. Run the following

```sh
$ git clone https://github.com/YOUR-GITHUB-PROFILE-NAME/shiplift.git
$ cd shiplift
```


### Please pay attention to
1. open an issue describing the feature/bug you wish to contribute first to start a discussion, explain why, what and how
2. use rustfmt, see below how to configure
3. try to write tests covering code you produce as much as possible, especially critical code branches
4. add notes/hightlights for the changelog in the pull request description


### Configuring rustfmt

Before submitting code in a PR, make sure that you have formatted the codebase
using [rustfmt][rustfmt]. `rustfmt` is a tool for formatting Rust code, which
helps keep style consistent across the project. If you have not used `rustfmt`
before, it is not too difficult.

If you have not already configured `rustfmt` for the
nightly toolchain, it can be done using the following steps:

**1. Use of the Nightly Toolchain**

Install the nightly toolchain. This will only be necessary as long as rustfmt
produces different results on stable and nightly.

```sh
$ rustup toolchain install nightly
```

**2. Add the rustfmt component**

Install the most recent version of `rustfmt` using this command:

```sh
$ rustup component add rustfmt --toolchain nightly
```

**3. Running rustfmt**

To run `rustfmt`, use this command:

```sh
$ cargo +nightly fmt --all
```

[rustfmt]: https://github.com/rust-lang-nursery/rustfmt


### Finding issues to fix
After you've forked and cloned our repo, you can find issues to work on by heading over to our [issues list](https://github.com/softprops/shiplift/issues)
