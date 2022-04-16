# s-expr

Rust library for S-expression like parsing and printing

* parser keeps track of spans, and representation (e.g. number base)
* number and decimal don't limit size
* only 1 dependency on `unicode-xid`


## S-expressions features

Extra features which are not in usual s-expressions (cannot be turned off):

* binary and hexadecimal number, when starting a number with the prefixes respectively `0b` or `0x`.
* `_` characters in number, e.g. `0xfedc__1240__abcd` or `100_000_000` to improve legibility

Currently unsupported:

* symbol with spaces
* negative literal integral and decimal, currently `-123` will be tokenized as the ident `-` followed by number `123`.
* scientific notation for decimal numbers `6.022e23` will be parsed as decimal `6.022` then ident `e`, then number `23`

There's lots of variant of S-expression, so the parser allow to parse various
different optional features that can be enabled/disabled depending on the user wishes:

* semi-colon line comment
* byte string of the format : `#8BADF00D#`
* braces `{ }` and bracket `[ ]` group, which behave like `( )` but provide
  equivalent grouping balancing check and flavor of grouping
