# govm

A simple Go version manager. Switch between Go versions with a single command.

## Why govm?

I got tired of manually downloading Go versions and fiddling with symlinks. Existing tools like `gvm` require installing a bunch of dependencies (Git, Mercurial, GCC...). I just wanted something that works.

govm is a single binary. No dependencies. Install it, and you're done.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/maneeshsagar/govm/main/install.sh | bash
```

Then restart your terminal.

## Usage

```bash
# See what's available
govm list-remote

# Install and switch to a version
govm use 1.22.0

# That's it
go version
```

### Per-project versions

Want different Go versions for different projects? Just create a `.go-version` file:

```bash
cd my-project
govm use 1.21.0 --local
```

Now every time you're in that directory, govm automatically uses Go 1.21.0.

### All commands

```
govm use <version>          Switch to a version (installs if needed)
govm use <version> --local  Set version for current project
govm install <version>      Just install, don't switch
govm versions               Show installed versions
govm list-remote            Show available versions
govm uninstall <version>    Remove a version
govm prune                  Clean up old versions
```

## How it works

govm uses shims - small scripts that intercept calls to `go` and `gofmt`. When you run `go build`, the shim figures out which Go version to use by checking:

1. `GOVM_VERSION` environment variable
2. `.go-version` file in current or parent directory
3. Global default (`~/.govm/version`)

Then it runs the actual Go binary from that version.

## Building from source

```bash
git clone https://github.com/maneeshsagar/govm.git
cd govm
cargo build --release
./install.sh
```

## Uninstall

```bash
rm -rf ~/.govm
```

And remove the govm lines from your `~/.zshrc` or `~/.bashrc`.

## Contributing

Found a bug? Have an idea? Check out [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT - do whatever you want with it.
