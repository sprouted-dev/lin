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

Tokens are stored in `~/.linear-cli/tokens.json` by default. To use the OS keychain instead:

```sh
lin login <your-api-key> --keyring
```

You can also provide a token via the `LINEAR_API_TOKEN` environment variable:

```sh
LINEAR_API_TOKEN=<token> lin issue list --team APP
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
lin issue create "Sprint task" --team APP --cycle current
lin issue create "Ship v2" --team APP --due-date 2026-09-30
lin issue edit ENG-123 --state "In Progress"
lin issue edit ENG-123 --cycle 42
lin issue edit ENG-123 --due-date 2026-09-30
lin issue edit ENG-123 --due 1w              # one week from today
lin issue state ENG-123
lin issue state ENG-123 "Done"
lin issue state ENG-123 --list
lin issue attachments list ENG-123
lin issue attachments add ENG-123 screenshot.png
lin issue attachments download ENG-123 -o /tmp/
lin issue attachments download ENG-123 --attachment-id f17b -o /tmp/
lin issue relate ENG-123 --to ENG-456                 # related (default)
lin issue relate ENG-123 --to ENG-456 --type blocks   # ENG-123 blocks ENG-456
lin issue relate ENG-123 --to ENG-456 --type blocked-by
lin issue relate ENG-123 --to ENG-456 --type duplicate
lin issue relations ENG-123
lin issue unrelate <relation-id>                      # id from `lin issue relations`
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
lin project view "My Project" --content
lin project create "My Project" --teams APP
lin project edit "My Project" --state started
lin project update list "My Project"
lin project update add "My Project" "Sprint update" --health onTrack
```

### Cycles

```sh
lin cycle list --team APP
lin cycle active --team APP
lin cycle show current --team APP
lin cycle show 42 --team APP
lin cycle create --team APP --starts 2026-04-07 --duration 1w --name "Sprint 1"
lin cycle create --team APP --starts 2026-04-07 --ends 2026-04-14
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

### Raw GraphQL

For anything the dedicated commands don't cover, `lin gql` runs a raw query or
mutation against the Linear API using your stored token. Output is always JSON.

```sh
# Inline query
lin gql 'query { viewer { id name email } }'

# With variables (JSON object)
lin gql 'query($id: String!) { issue(id: $id) { identifier title } }' \
  --variables '{"id": "ENG-123"}'

# Read the document from a file, or - for stdin
lin gql @query.graphql --variables @vars.json
cat query.graphql | lin gql -
```

### Identifier Resolution

Most flags accept human-readable names instead of UUIDs:

- `--team` accepts a team name (e.g., `Engineering`), key (e.g., `APP`), or UUID
- `--assignee` accepts a user name, email, `me`, or UUID
- `--project` accepts a project name or UUID
- `--cycle` accepts a cycle name, number, or `current`
- Issue identifiers like `ENG-123` are resolved automatically

### Global Flags

```sh
lin --json issue list --team APP    # Raw JSON output
lin -v issue list --team APP        # Verbose error output (shows response body on failures)
lin -w my-workspace issue list      # Target a specific workspace
```

Run `lin --help` for the full command reference.

## Contributing

```sh
just setup   # configure git hooks
just check   # format, lint, test
```

## License

[MIT](LICENSE)
