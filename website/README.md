# Doux Website

Documentation and playground for the Doux synthesizer.

## Credits

Doux is a Rust port of [Dough](https://codeberg.org/uzu/dough), originally written in C by [Felix Roos](https://codeberg.org/froos). Dough is part of the [TidalCycles](https://tidalcycles.org) ecosystem. Consider [supporting the project](https://opencollective.com/tidalcycles).

## Architecture

- **SvelteKit** with static adapter for GitHub Pages deployment
- **mdsvex** for markdown-based documentation in `src/content/`
- **WASM module** (`static/doux.wasm`) — the synthesizer engine
- **COI service worker** for Cross-Origin Isolation — browsers require `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers to enable `SharedArrayBuffer`, which is needed for audio worklets. Since GitHub Pages doesn't allow custom headers, the service worker injects them client-side.

### Directory Structure

```
src/
├── content/     # Markdown documentation files
├── lib/         # Shared components and utilities
├── routes/
│   ├── native/     # Native build downloads
│   ├── reference/  # API reference pages
│   └── support/    # Support pages
└── app.html     # HTML shell
```

## Development

```bash
pnpm install
pnpm dev
```

## Build

```bash
pnpm build
```

Output goes to `build/`.

## Deployment

Automatic via GitHub Actions on push to `main`. The workflow:

1. Builds the static site with `pnpm build`
2. Deploys to GitHub Pages

See `.github/workflows/deploy.yml` for details.

## License

AGPL-3.0
