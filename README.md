# flake-iter

A convenient tool for building all of your Nix stuff.

## Build all derivations

```shell
flake-iter build
```

Add the `--verbose` flag for debug-level logging and output piped from each `nix build` invocation.

## Output systems list

```shell
flake-iter systems
```

This writes a list of the form below to the `file` at `$GITHUB_OUTPUT`:

```json
[
  {
    "nix-system": "aarch64-darwin",
    "runner": "macos-latest"
  }
]
```

It's intended for use only in GitHub Actions runs.

The `systems` command maps Nix systems to runners.
Here's the default mapping:

```json

{
  "x86_64-linux": "ubuntu-latest",
  "x86_64-darwin": "macos-latest",
  "aarch64-darwin": "macos-latest"
}
```

You can provide your own custom mapping using the `--runner-map` option:

```shell
flake-iter systems \
  --runner-map '{"x86_64-linux":"chonky-linux-box","x86_64-darwin":"chonky-macos-box"}'
```
