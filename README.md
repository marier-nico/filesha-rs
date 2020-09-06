# Filesha-rs

This project is a fun experiment to better understand how to work with the Rocket web framework and
Diesel, Rust's currently most popular ORM.

# Motivation

The idea behind the project is to remain as simple as possible, while providing the necessary features
to users with good performance. Initial development was triggered by the complexity and bad performance
of NextCloud's file hosting and sharing features as well as other overly complex services. Most other
options are for large businesses with many data sources and try to be distributed to accommodate for
the scale of an enterprise.

The approach with this project is to provide all the valuable features of a file-sharing service, but
in an easy to use and compact package.

# Ambitions

This project is not meant to be used in production by anyone, its aim is to experiment an have fun while
building something concrete. However, this may change over time if and when features are added and the
existing ones improve.

# API Methods

- `File`: https://documenter.getpostman.com/view/4572944/TVCiSkkP
- `User`: https://documenter.getpostman.com/view/4572944/TVCiSkpf

# Features

## Performance

Filesha-rs was built with performance in mind and efforts were made to deliver as much performance
as possible. The decision to implement the API in Rust also partly came from the desire to offer solid
performance and reliability.

## Simplicity

This service is very self-contained, making it straight-forward to setup. There is no additional
database to setup, no external services to use, no networking for clusters, no distributed hosts for
load-balancing. None of that. This may seem to go against the idea of providing performance, but
these features are not needed for a small amount of users, so simplicity can be offered with no noticeable
performance difference for the intended workloads.

# Contributing

This project is built on top of the [Rocket](https://rocket.rs/) web framework, which makes extensive
use of Rust nightly features. As such, this project runs on Rust nightly. Please see Rocket's docs
for instructions on how to get started.

The next step is to build the project for the first time with `cargo build`. This will download all
dependencies and compile the code.

Then, install the Diesel CLI, which will allow you to properly setup your database with an up-to-date
schema : `cargo install diesel_cli`.

Finally, run `diesel migration run` to run all migrations to bring your database up to date.