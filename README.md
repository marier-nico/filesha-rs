# Filesha-rs

This project aims to provide an easy-to-use and easy-to-setup file-hosting and file-sharing service.

# Motivation

The idea behind the project is to remain as simple as possible, while providing the necessary features
to users with good performance. What initially sparked development in this was dissatisfaction at
NextCloud's performance for file-hosting as well as other overly-complex services. Most other options
are for large businesses with many data sources and try to be distributed to accommodate for the scale
of an enterprise.

The approach with this project is to provide all the valuable features of a file-sharing service, but
in an easy to use and compact package.

# Features

### Performance

Particular attention has been paid to offer good performance, though part of the performance simply
comes from the fact that Rust was the chosen implementation language instead of PHP or Python, for
example.

## Simplicity

This service is very self-contained, making it straight-forward to setup. There is no additional
database to setup and create users for, no networking for clustering setups, no multiple hosts for
load-balancing. None of that. This may seem to go against the idea of providing performance, but
these features are not needed when used by few users, so simplicity can be offered with no noticeable
performance difference for the target workloads.

# Contributing

This project is built on top of the [Rocket](https://rocket.rs/) web framework, which makes extensive
use of Rust nightly features. as such, this project runs on nightly Rust. Please see Rocket's docs
for instructions on how to enable those features.

The next step is to build the project for the first time with `cargo build`. This will download all
dependencies and compile the code.

Then, install the Diesel CLI, which will allow you to properly setup your database with an up-to-date
schema : `cargo install diesel_cli`.

Finally, run `diesel migration run` to run all migrations to bring your database up to date.