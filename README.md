# create-sandybridge-app

A CLI tool for scaffolding new sandybridge.io projects.

## Installation

```bash
cargo install --path .
```

Or build and symlink manually:

```bash
cargo build --release
ln -s $(pwd)/target/release/create-sandybridge-app ~/.cargo/bin/create-sandybridge-app
```

## Usage

```bash
create-sandybridge-app <project-name> [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `--docs` | Include MDX documentation system |
| `--no-auth` | Skip next-auth authentication setup |
| `--db <type>` | Database type: `postgres` (default) or `none` |
| `--path <dir>` | Output directory (default: `./<project-name>`) |
| `--no-git` | Skip git initialization |
| `--no-install` | Skip npm install |
| `--template <url>` | Custom template repository URL |
| `-y, --yes` | Accept all defaults without prompting |

### Examples

**Create a full-featured app (default):**
```bash
create-sandybridge-app my-app
```

**Create an app with documentation:**
```bash
create-sandybridge-app my-docs-app --docs
```

**Create a minimal app without auth or database:**
```bash
create-sandybridge-app my-simple-app --no-auth --db none
```

**Non-interactive mode (CI/scripts):**
```bash
create-sandybridge-app my-app -y
```

## What's Included

By default, projects include:

- **Next.js 15** with App Router and Turbopack
- **TypeScript** configuration
- **Tailwind CSS 4** with theme support
- **shadcn/ui** components via `@thesandybridge/ui`
- **Theme system** via `@thesandybridge/themes` (light/dark mode, cross-subdomain cookies)
- **next-auth v5** with GitHub OAuth (unless `--no-auth`)
- **PostgreSQL** database setup (unless `--db none`)
- **React Query** for data fetching

### Optional Features

- **MDX Documentation** (`--docs`): Adds a docs system with Shiki syntax highlighting

## Environment Variables

When auth is enabled, a `.env` file is generated with random secrets:

```env
DATABASE_URL=postgresql://user:password@localhost:5432/dbname
AUTH_SECRET=<random-64-char-hex>
AUTH_GITHUB_ID=your-github-oauth-app-id
AUTH_GITHUB_SECRET=your-github-oauth-app-secret
JWT_SECRET=<random-64-char-hex>
```

## Template Repository

The default template is fetched from:
https://github.com/thesandybridge/app-template

You can use a custom template with `--template <url>`.
