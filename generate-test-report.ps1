#!/usr/bin/env pwsh

$base_rustflags = ''
# Extra cargo args, e.g. @('--features', 'foo'). Empty by default.
# (An array, not a string, so PowerShell passes each token as its own argument.)
$cargo_features = @()

$env:RUSTFLAGS = "$base_rustflags --allow=warnings -Cinstrument-coverage"

# build-* ones are not parsed by grcov
$env:LLVM_PROFILE_FILE = 'profiling/build-%p-%m.profraw'
cargo build @cargo_features --all-targets --locked --workspace

# cleanup old values
Get-ChildItem -Path . -Recurse -Filter '*.profraw' -File | Remove-Item -Force

# different from the `cargo build` ones
$env:LLVM_PROFILE_FILE = 'profiling/profile-%p-%m.profraw'
cargo nextest run --profile ci --no-fail-fast @cargo_features --all-targets --workspace

grcov $(Get-ChildItem -Recurse -Filter 'profile-*.profraw' -Path . | ForEach-Object { $_.FullName }) `
  --binary-path ./target/debug/ `
  --branch `
  --excl-br-line '^\s*((debug_)?assert(_eq|_ne)?!)' `
  --excl-br-start 'mod tests \{' `
  --excl-line '(#\[derive\()|(^\s*.await[;,]?$)' `
  --excl-start 'mod tests \{' `
  --ignore-not-existing `
  --keep-only 'crates/**' `
  --llvm `
  --output-path ./reports/ `
  --output-type lcov `
  --source-dir .
