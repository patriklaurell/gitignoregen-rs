# gitignoregen-rs

A minimal interactive CLI wrapper around [Toptal gitignore](https://www.toptal.com/developers/gitignore). Written in rust using [`dialoguer`](https://github.com/console-rs/dialoguer), [`clap`](https://github.com/clap-rs/clap) and [`reqwest`](https://github.com/seanmonstar/reqwest).

## Installation

```
cargo install gitignoregen
```

## Usage

Generate a new `.gitignore` file in the current working directory with

```
gitignoregen
```

and append to the existing with

```
gitignoregen append
```

![Usage](usage.gif)
