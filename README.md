# displayctl

List and toggle macOS displays from the command line.

## Installation

### Prebuilt binaries

Prebuilt binaries for Apple Silicon (`arm64`) and Intel (`x86_64`) Macs are attached to each [GitHub Release](https://github.com/dnlzro/displayctl/releases). Download and extract the tarball for your architecture:

```sh
curl -LO https://github.com/dnlzro/displayctl/releases/download/v0.1.0/displayctl-v0.1.0-arm64.tar.gz
tar xzf displayctl-v0.1.0-arm64.tar.gz
sudo mv displayctl-v0.1.0-arm64/displayctl /usr/local/bin/
```

Or install with Homebrew or Nix below.

### Homebrew

```sh
brew install dnlzro/displayctl/displayctl
```

Or install the latest development version:

```sh
brew install dnlzro/displayctl/displayctl --HEAD
```

### Nix

**Using flakes:**

```sh
nix run github:dnlzro/displayctl
```

Or add it to your flake inputs and install via `packages` or `home.packages`.

**Without flakes:**

```sh
nix-env -f https://github.com/dnlzro/displayctl/archive/main.tar.gz -iA default
```

Or clone the repository and run:

```sh
nix-build
nix-env -i ./result
```

## Usage

```sh
displayctl list
displayctl disable <uuid>
displayctl enable <uuid>
```

For example:

```sh
$ displayctl list
NAME              STATUS  MODE       NATIVE     REFRESH  UUID
Built-in Display  active  1440x900   2880x1800  60.00Hz  37D8832A-2D66-02CA-B9F7-8F30A301B230
External Display  active  1920x1080  -          60.00Hz  E4DB018A-E193-64D6-0857-D6964E3302DB

$ displayctl disable E4DB018A-E193-64D6-0857-D6964E3302DB
```

> [!WARNING]
> If you disable a display, you may need to unplug and replug it or restart your Mac to bring it back. Re-enabling via `displayctl enable <uuid>` does not work on all setups.
