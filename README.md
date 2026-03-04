# lin

A fast CLI for [Linear](https://linear.app).

## Install

### Homebrew (macOS & Linux)

```sh
brew install sprouted-dev/tap/lin
```

### Cargo

```sh
cargo install lin-cli
```

### From source

```sh
git clone https://github.com/sprouted-dev/lin.git
cd lin
cargo install --path .
```

## Setup

1. Create a [Linear API key](https://linear.app/settings/api).

2. Log in:

```sh
lin login --token <your-api-key> --name <workspace-name>
```

## Usage

```sh
# View an issue
lin issue view LIN-42

# Search issues
lin issue search "bug in auth"

# Create an issue
lin issue create --title "Fix login" --team-id <TEAM_ID>

# List projects
lin project list

# List teams
lin team list

# Switch workspace
lin workspace set my-workspace
```

Run `lin --help` for the full command reference.

## Contributing

```sh
just setup   # configure git hooks
just check   # format, lint, test
```

## License

[MIT](LICENSE)
