# roommates

![Rust](https://github.com/jonahweissman/roommates/workflows/Rust/badge.svg)
[![Rust Docs](https://github.com/jonahweissman/roommates/workflows/Rust%20Docs/badge.svg)](https://jonahweissman.github.io/roommates/roommates/index.html)
---
This is a simple rust library for splitting bills between multiple people. Under normal circumstances,
bills can be split evenly between all roommates, but when one or more roommates is gone for an extended
period of time, this is no longer fair. Instead of making the few roommates who remain shoulder the
entire cost, this library:
- uses the history of bills to build a linear model to estimate costs based on occupancy
- predicts the cost if the occupancy had been zero (the "fixed cost")
- divides the fixed cost evenly among all roommates
- charges the present roommates based on the proportion of the billing interval for which they were present

Check the [examples](/examples) folder for a simple command line interface.

Does not currently handle rounding errors, but will soon.

### Documentation

Hosted on GitHub Pages: https://jonahweissman.github.io/roommates/roommates/index.html

### License

This project is licensed under either the MIT License or the Apache-2.0 License,
at your option.
