# 0.4.0 (unreleased)

This release brings a number of breaking changes, all hopefully considered to be *good* :), and some new process to help track future changes. Some notable changes are listed below. The best source of truth will be the updated rustdocs.

* removed public {Entity}Builder.new() constructors. Use `{Entity}.builder()` interfaces to construct these instead [#125](https://github.com/softprops/shiplift/pull/125)
* repo labels are now optional [#102](https://github.com/softprops/shiplift/pull/102)
* interlacing tty [#101](https://github.com/softprops/shiplift/pull/101)
* migrate to serde and update struct field names to line up with rust's snake_case conventions [#100](https://github.com/softprops/shiplift/pull/100)
* update `byteorder` and `flate2` dependencies [#99](https://github.com/softprops/shiplift/pull/99)
* add `Type`, `Action`, `Actor` [#98](https://github.com/softprops/shiplift/pull/98)
* add representations of `NetworkSettings#Networks` [#97](https://github.com/softprops/shiplift/pull/97)
* remove `MemoryStat#{swap,total_swap}`[#96](https://github.com/softprops/shiplift/pull/96)
* make `Image#RepoTags` an Option type [#95](https://github.com/softprops/shiplift/pull/95)
* add `ContainerDetails#Name` [#93](https://github.com/softprops/shiplift/pull/93)
* make unix socket support optional to enable windows use [#92](https://github.com/softprops/shiplift/pull/92)
* upgrade to `hyper@0.12` [#91](https://github.com/softprops/shiplift/pull/91)
* change `SearchResult#is_trusted` to `SearchResult#is_automated` [#89](https://github.com/softprops/shiplift/pull/89)
* add container builder option for memory [#86](https://github.com/softprops/shiplift/pull/86)
* allow `HostConfig#MemorySwap` to be negative [#87](https://github.com/softprops/shiplift/pull/87)

# 0.3.2
* upgraded to hyper 0.10
* added interfaces for container log_driver, restart_policy
* added container exec interface
* added docker network interfaces

# 0.3.1

* added support for `CapAdd` on `ContainerOptions` [#32](https://github.com/softprops/shiplift/pull/32)
* changed representation of `OnBuild` from a `String` to `Ve     r exists in remote api [#20](https://github.com/softprops/shiplift/pull/20)
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
