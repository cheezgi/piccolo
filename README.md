
# piccolo

Piccolo is a small, light, high-pitched scripting language (eventually)
intended for embedding in Rust projects.

*currently requires nightly: waiting for [Any::type_id](https://github.com/rust-lang/rust/issues/27745)*

## TODO

* [X] lexer
* [X] parser
* [X] support for most stuff
    * [X] value types
    * [X] functions
        * [X] normal function definitions
        * [X] native functions
    * [X] data
        * [X] methods
        * [X] instance variables
* [ ] module system
    * [X] builtin (sys, io, string, ...)
    * [ ] functions (require, export)
* [ ] lua-style userdata
* [ ] array indexing
* [ ] document/polish API
    * [ ] annotate heavily
* [ ] varargs for normal functions

