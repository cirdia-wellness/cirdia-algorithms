### Documentation README — How to maintain algorithm docs

- Place algorithm docs in ```docs/``` or each crate’s ```README.md```.

- Keep docs and code in sync: when a public API changes, update the matching docs section.

- Testing: add unit test vectors in ```tests/``` inside each crate; include a ```tests/data/``` folder with sample CSV or JSON.

- Release process: when algorithms change, increment semantic version in ```Cargo.toml``` and add short release notes to ```docs/RELEASE_NOTES.md```.
