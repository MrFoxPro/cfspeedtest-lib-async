# cfspeedtest - Unofficial CLI for [speed.cloudflare.com](https://speed.cloudflare.com)
[![Build](https://github.com/code-inflation/cfspeedtest/actions/workflows/CI.yml/badge.svg?branch=master)](https://github.com/code-inflation/cfspeedtest/actions/workflow[![CI](https://github.com/code-inflation/cfspeedtest/actions/workflows/CI.yml/badge.svg)](https://github.com/code-inflation/cfspeedtest/actions/workflows/CI.yml)s/CI.yml)

## TODO
- [ ] Dynamic payload sizing depending on network speed
- [ ] Consider server processing time in measurements
- [X] CLI arguments (~~nr of tests~~, ~~payload sizes~~, ~~verbosity~~)
- [X] Clean up output
- [X] Boxplot for measurements
- [ ] Asciinema recording in readme
- [ ] Publish crate
- [ ] Install and usage instructions
- [ ] Add Serde to provide CSV/ TSV ... output

## Development
### Logging
Set the log level using the `RUST_LOG` env var:  
```sh
RUST_LOG=debug cargo run
```
### Release
#### On GitHub
Release builds are published automatically using github actions. They are triggered when a git tag in the format `v[0-9]+.*` is pushed.
```sh
git tag v1.0.0
git push origin v1.0.0
```
#### On crates.io
1. Update `cfspeedtest` version in `Cargo.toml`
2. `cargo publish --dry-run`
3. Verify contents using `cargo package --list`
4. Upload to crates.io `cargo publish`
