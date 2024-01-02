_default:
  just --choose

# Build all crates and other parts of the project
build:
  cargo build --release
  bun install
  cd book/highlight && bun run build
  cd book && just build
  cd vscode-extension && bun run package

# Run the benchmarks
bench:
  cargo bench -p mabo-benches

# Run clippy over all crates, testing every feature combination
check:
  cargo hack clippy --workspace --feature-powerset --no-dev-deps

# Format the code of all Rust crates
fmt:
  cargo +nightly fmt --all

# Run snapshot tests and review any updates
snapshots:
  cargo insta test --workspace --all-features \
    --test-runner nextest \
    --unreferenced delete \
    --review

# Start up the local server for the book
@book:
  cd book && just dev

# Check all links of the crates and book
linkcheck:
  cd book && just build
  lychee --cache --max-cache-age 7d \
    --exclude https://github\.com/dnaka91/mabo \
    'book/src/**/*.md' \
    'book/book/**/*.html' \
    'crates/**/*.rs'

# Install the LSP server into the local system
install-lsp:
  cargo install --path crates/mabo-lsp --offline --debug

# Install the VSCode extension into VSCodium
install-vscodium: install-lsp
  bun install
  cd vscode-extension && \
    bun run package && \
    codium --install-extension mabo-*.vsix
