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
lin login <your-api-key>
```

To associate the token with a named workspace:

```sh
lin login <your-api-key> --name my-workspace
```

## Usage

### Issues

```sh
lin issue view ENG-123
lin issue search "bug in auth"
lin issue search "login" --team APP --status "In Progress"
lin issue list --team APP
lin issue list --assignee me --status "In Progress"
lin issue me
lin issue me --status "Todo"
lin issue create "Fix login" --team APP
lin issue create "Fix login" --team APP --assignee me --priority 2
lin issue edit ENG-123 --state "In Progress"
lin issue state ENG-123
lin issue state ENG-123 "Done"
lin issue state ENG-123 --list
lin issue attachments ENG-123
```

### Comments

```sh
lin comment view ENG-123
lin comment add ENG-123 "Looks good"
lin comment edit <comment-id> "Updated comment"
```

### Projects

```sh
lin project list
lin project view "My Project"
lin project create "My Project" --teams APP
lin project edit "My Project" --state started
lin project update list "My Project"
lin project update add "My Project" "Sprint update" --health onTrack
```

### Cycles

```sh
lin cycle list --team APP
lin cycle active --team APP
```

### Initiatives

```sh
lin initiative list
lin initiative view <initiative-id>
```

### Teams

```sh
lin team list
```

### Users

```sh
lin user list
```

### Labels

```sh
lin label list
lin label list --team APP
lin label create "Bug" --team APP
```

### Workspaces

```sh
lin workspace current
lin workspace list
lin workspace set my-workspace
```

Use the `-w` flag to target a specific workspace:

```sh
lin -w my-workspace issue view ENG-123
```

### Identifier Resolution

Most flags accept human-readable names instead of UUIDs:

- `--team` accepts a team name (e.g., `Engineering`), key (e.g., `APP`), or UUID
- `--assignee` accepts a user name, email, `me`, or UUID
- `--project` accepts a project name or UUID
- Issue identifiers like `ENG-123` are resolved automatically

Run `lin --help` for the full command reference.

## Contributing

```sh
just setup   # configure git hooks
just check   # format, lint, test
```

## License

[MIT](LICENSE)
