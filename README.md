# Codeforces tester

Run test for your codeforces solutions

## Installation

Install [rustup](https://rustup.rs), clone this repository, then from inside this repo:

```sh
cargo install --path=cli
```

## Configuration

See [example config](docs/cdf.toml) for explanation of config format.

## Usage

Copy [example config](docs/cdf.toml) to the folder where you will run the tests, and change for your use.

To test task with specified id, run:

```sh
cdf test [id]
```
