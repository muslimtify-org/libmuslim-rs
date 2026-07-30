# Releasing

Publishing to crates.io is intentionally manual. CI verifies that a release
*would* work (`cargo publish --dry-run` on every push), and the tag workflow
builds and attaches artifacts, but the irreversible step is always run by a
human.

> A published version can never be replaced or deleted — only yanked, which
> hides it from new resolutions but leaves it downloadable forever. Treat the
> `cargo publish` step as permanent.

## One-time setup

```bash
cargo login          # stores a crates.io API token in ~/.cargo/credentials.toml
```

Generate the token at <https://crates.io/settings/tokens>, scoped to
`publish-new` for the first release and `publish-update` afterwards.

No repository secret is needed: nothing in CI publishes on your behalf.

## Steps

1. **Bump the version** in `Cargo.toml`. The tag workflow refuses to build if
   the tag and manifest disagree, so this comes first.

2. **Update `CHANGELOG.md`** — move entries out of `[Unreleased]` into a new
   version section and refresh the link definitions at the bottom.

3. **Refresh `Cargo.lock`.** It is committed, and CI builds with `--locked`, so
   a version bump that leaves it stale fails every job:

   ```bash
   cargo update --workspace   # rewrites only this crate's own version entry
   ```

4. **Run the checks locally.** These mirror CI, so a failure here is a failure
   there:

   ```bash
   cargo fmt --all --check
   cargo clippy --locked --all-targets --all-features -- -D warnings
   cargo test --locked --all-targets --all-features
   cargo test --locked --doc --all-features
   RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
   ```

5. **Inspect what will actually ship.** The vendored C sources in `include/`
   are required at build time, so confirm they are in the package — a consumer
   whose download lacks them gets a build-script failure, not a clear error:

   ```bash
   cargo package --locked --list --all-features | grep include/
   cargo publish --locked --dry-run --all-features
   ```

6. **Commit and tag.** `Cargo.lock` is tracked, so include it:

   ```bash
   git commit -am "release: v0.1.0"
   git tag -a v0.1.0 -m "v0.1.0"
   git push origin main --follow-tags
   ```

   Pushing the tag triggers `.github/workflows/release.yml`, which re-verifies
   the version, builds artifacts for Linux, macOS (Intel and Apple Silicon),
   and Windows, and creates the GitHub Release with checksums.

7. **Wait for the tag workflow to go green**, then publish:

   ```bash
   cargo publish --locked --all-features
   ```

8. **Confirm the result** at <https://crates.io/crates/libmuslim-rs> and
   <https://docs.rs/libmuslim-rs>. docs.rs builds independently of CI and can
   fail on its own — check that the build succeeded.

## If something is wrong after publishing

```bash
cargo yank --version 0.1.0            # stop new dependents resolving to it
cargo yank --version 0.1.0 --undo     # reverse the yank
```

Yanking does not remove the version. Fix forward with a new patch release.
