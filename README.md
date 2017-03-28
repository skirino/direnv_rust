# direnv_rust

Clone of [direnv](https://github.com/direnv/direnv) in [Rust](https://www.rust-lang.org).
Besides the core features (auto-loading of env vars, security checks), most of fancy features of direnv are missing.
However this supports .env files in nested directories.

# Usage

Put the following code into your `.zshrc`.

```sh
_direnv_rust_hook() {
  eval "$(direnv_rust)"
}
add-zsh-hook chpwd _direnv_rust_hook
_direnv_rust_hook # invoke the hook once on startup of zsh
```

Place `.env` file in some directory.
Currently supported directives in `.env` files are:

- `set NAME VALUE`
- `unset NAME`
- `append NAME VALUE`
- `prepend NAME VALUE`
