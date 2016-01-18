# 0.2.1 (unreleased)

* removed `Body` type with a preference for `Into<hyper::client::Body>`
* implemented `Image.build`
* renamed `Image.create` to `Image.pull` to avoid confusion with `Image.build` and added `PullOptions` argument and return type of iterable `PullOutput`

# 0.2.0

* many breaking changes required to make interfaces consistent, idomatic, and future friendly
* better error representations
* improve (remove) mut in interfaces where it was no longer needed
* update deps
* refactor to use [hyperlocal](https://github.com/softprops/hyperlocal) for better unix domain socket transport support
* expose Config env as a map instead of a vec of strings
* add support for export containers and image

# 0.1.2

* Update dependencies

# 0.1.1

* Added events interface

# 0.1.0

* Initial release
