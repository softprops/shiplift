# 0.3.0 (unreleased)

* upgraded [hyper](https://github.com/hyperium/hyper/) from 0.7 to 0.9 [#13](https://github.com/softprops/shiplift/pull/29)
* upgraded [hyperlocal](https://github.com/softprops/hyperlocal) dependency from 0.1 to 0.2
* added documentation updates [#14](https://github.com/softprops/shiplift/pull/14)
* added container deletion interface that takes options [#15](https://github.com/softprops/shiplift/pull/15)
* return Err rather than panicing on IO errors communicating with the client [#16](https://github.com/softprops/shiplift/pull/16)
* expose `container.id()` to Container service interface [#17](https://github.com/softprops/shiplift/pull/17/files)
* add events filter to EventsOptionsBuilder [#18](https://github.com/softprops/shiplift/pull/18)
* remove ExecDriver field of container details which no longer exists in remote api [#20](https://github.com/softprops/shiplift/pull/20)
* remote ExecutionDriver from info which no longer exists in remote api
* add more options to ContainerOptions creation interface [#23](https://github.com/softprops/shiplift/pull/23)
* fix volume parameter [#24](https://github.com/softprops/shiplift/pull/24)
* make id and event optional for event reps to accommodate new network type events [#26](https://github.com/softprops/shiplift/pull/26)
* set a host header for unix domain socket interface due to a golang impl detail in newer versions of the docker daemon [#27](https://github.com/softprops/shiplift/pull/27)
* implement std error trait for shiplift's error type to make it play nicely with other error handling tools  [#28](https://github.com/softprops/shiplift/pull/28)
* changed `failcnt` field on `stats.memory_stats` to `Option<u64>` as it is not always returned in newer versions of the docker remote api.
* removed `size` field in image listings as its returned as -1 in newer remote api versions

# 0.2.1

* removed `Body` type with a preference for `Into<hyper::client::Body>`
* implemented `Image.build`
* renamed `Image.create` to `Image.pull` to avoid confusion with `Image.build` and added `PullOptions` argument and return type of iterable `PullOutput`

# 0.2.0

* many breaking changes required to make interfaces consistent, idiomatic, and future friendly
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
