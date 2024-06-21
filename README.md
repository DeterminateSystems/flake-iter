# flake-iter

A convenient tool for building all of your Nix stuff.

## Build all derivations

The `build` command determines which of your flake's outputs are derivations and builds them all:

```shell
flake-iter build
```

Add the `--verbose`/`-v` flag for debug-level logging and output piped from each `nix build` invocation.

```shell
flake-iter build --verbose
```

You can specify a directory different from the current directory using the `--directory`/`-d` option:

```shell
flake-iter build --directory ./my-dir
```

You can also specify a Nix system to build for:

```shell
flake-iter build --system myarch-myos
```

If not specified, `flake-iter` detects your current system.

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
