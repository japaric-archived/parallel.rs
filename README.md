# DEPRECATED

Reason: Now that's safe to send references/slices, this library provides very
little advantage over directly calling `thread::scoped`.

This library won't receive further fixes nor upgrades.

---

[![Build Status][status]](https://travis-ci.org/japaric/parallel.rs)

# `parallel.rs`

(Hopefully) Safe fork-join parallelism abstractions.

Implements the `divide` and `execute` functions proposed in Niko's
[blog][blog].

# [Documentation][docs]

# License

parallel.rs is dual licensed under the Apache 2.0 license and the MIT license.

See LICENSE-APACHE and LICENSE-MIT for more details.

[blog]: http://smallcultfollowing.com/babysteps/blog/2013/06/11/data-parallelism-in-rust
[docs]: http://japaric.github.io/parallel.rs/parallel/
[status]: https://travis-ci.org/japaric/parallel.rs.svg?branch=master
