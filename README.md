pgen
====

Rust parser generator project-- WIP

Check out the example grammar at [src/this.rs](src/this.rs) and its generated output at [wtf/src/lib.rs](wtf/src/lib.rs)

Roadmap:

- [ ] Lexer generation
  - [ ] [Contextual scanning](https://www-users.cs.umn.edu/~evw/pubs/vanwyk07gpce/vanwyk07gpce.pdf)
- [ ] Parser generation
  - [x] SLR table
  - [ ] Lookaheads (weekend of 2020-01-03)
  - [ ] Error reporting
- [ ] Fancy things
  - [ ] Custom grammar file format
  - [ ] Parametric rules
  - [ ] **Really** fancy error reporting
  - [ ] Generate code for different backends...?
- [ ] Refactor...
- [ ] Documentation...

Contact
-------

License: MIT

Author: Michael Zhang