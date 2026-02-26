# nullslate

CLI for the nullslate dev tooling ecosystem.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
nullslate <command> [options]
# or use the short alias:
ns <command> [options]
```

### Commands

#### `init` — Scaffold a new project

```bash
nullslate init <project-name> [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--docs` | Include MDX documentation system |
| `--no-auth` | Skip Auth.js authentication setup |
| `--db <type>` | Database type: `postgres` or `none` (default) |
| `--path <dir>` | Output directory (default: `./<project-name>`) |
| `--no-git` | Skip git initialization |
| `--no-install` | Skip npm install |
| `--template <url>` | Custom template repository URL |
| `-y, --yes` | Accept all defaults without prompting |

### Examples

**Interactive mode:**
```bash
nullslate init my-app
```

**With all features:**
```bash
ns init my-app --docs --db postgres
```

**Non-interactive (CI):**
```bash
nullslate init my-app -y
```

## What's Included

Scaffolded projects include:

- **Vite** with React 19 and TanStack Router
- **TypeScript**
- **Tailwind CSS 4** with theme support
- **shadcn/ui** components
- **TanStack React Query** for server state

### Optional Features

- **Authentication**: Auth.js with GitHub OAuth
- **Documentation**: MDX docs system
- **Database**: PostgreSQL via `pg`

## Environment Variables

When auth is enabled, a `.env` file is generated:

```env
AUTH_SECRET=<random-64-char-hex>
AUTH_GITHUB_ID=
AUTH_GITHUB_SECRET=
```
