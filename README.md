# ormx

Lightweight macros for [sqlx](https://github.com/launchbadge/sqlx)

[![Crate](https://img.shields.io/crates/v/ormx.svg)](https://crates.io/crates/ormx)
[![API](https://docs.rs/ormx/badge.svg)](https://docs.rs/ormx)

## Getting started

Add ormx and sqlx to your `Cargo.toml`

```toml
[dependencies.ormx]
version = "0.5"
features = ["mysql"]

[dependencies.sqlx]
version = "0.5"
default-features = false
features = ["macros", "mysql", "runtime-tokio-rustls"]
```

Right now, ormx supports mysql/mariadb and postgres.

## What does it do?

ormx provides macros for generating commonly used sql queries at compile time.
ormx is meant to be used together with sqlx. Everything it generates uses `sqlx::query!` under the hood, so every generated query will be checked against your database at compile time.

## What doesn't it do?

ormx is not a full-fledged ORM nor a query builder. For everything except simple CRUD, you can always just use sqlx.

## Roadmap

see [TODO](TODO.tasks)

## [Example](https://github.com/NyxCode/ormx/tree/master/example/src/main.rs)

## Features

- `mysql` -  enable support for mysql/mariadb
- `postgres` - enable support for postgres
