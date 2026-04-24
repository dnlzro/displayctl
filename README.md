# displayctl

List and toggle macOS displays from the command-line.

## Installation

Prebuilt binaries for Apple Silicon (`arm64`) and Intel Macs are attached to each [GitHub Release](https://github.com/dnlzro/displayctl/releases).

Or install with Homebrew:

```sh
brew install dnlzro/tap/displayctl
```

Or use Nix:

```sh
nix run github:dnlzro/displayctl
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
