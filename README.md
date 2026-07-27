# rusty-todos

A simple todo-list web app built with Rust,
[Topcoat](https://github.com/tokio-rs/topcoat) and
[Toasty](https://github.com/tokio-rs/toasty).

## Run

Todos are persisted to a local sqlite database that is not checked in, so create
it first:

```
$ just migrate
```

Then start the dev server:

```
$ just dev
```

Then open http://localhost:3000.
