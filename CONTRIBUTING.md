# Contributing to govm

Thanks for considering contributing! Here's how you can help.

## Found a bug?

1. Check if it's already reported in [Issues](https://github.com/maneeshsagar/govm/issues)
2. If not, open a new issue with:
   - What you expected to happen
   - What actually happened
   - Steps to reproduce
   - Your OS and architecture (`uname -a`)

## Want to add a feature?

Open an issue first. Let's discuss if it fits the project before you spend time coding.

govm aims to be simple. We intentionally don't have features like:
- Building Go from source
- Package sets / isolated environments
- Windows support (PRs welcome though!)

## Making changes

```bash
# Fork and clone
git clone https://github.com/maneeshsagar/govm.git
cd govm

# Create a branch
git checkout -b my-fix

# Make your changes
# ...

# Run tests
cargo test

# Make sure it builds
cargo build --release

# Test manually
./target/release/govm --help
```

## Code style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix any warnings
- Keep it simple - readable code > clever code

## Pull requests

1. Keep PRs focused - one feature or fix per PR
2. Update tests if needed
3. Update README if you're changing user-facing behavior

## Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Questions?

Open an issue. I'm happy to help!
