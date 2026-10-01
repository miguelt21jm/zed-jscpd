# zed-jscpd

Runs [jscpd](https://github.com/kucherenko/jscpd) as a language server in Zed, so duplicated code shows up as diagnostics while you edit.

## Install

Not on the Zed extension registry yet. Clone this repo, then in Zed run `zed: install dev extension` and pick the folder.

The extension uses `jscpd` from your PATH if it's there (needs 5.4.0 or later). If not, it downloads the latest release from GitHub.

## Settings

Everything goes under `lsp.jscpd` in your Zed settings:

```json
{
  "lsp": {
    "jscpd": {
      "binary": {
        "path": "/path/to/jscpd",
        "arguments": ["--min-tokens", "40"]
      },
      "settings": {
        "lsp": { "complexity": { "enabled": true } }
      }
    }
  }
}
```

`settings` takes the same keys as `.jscpd.json`. A `.jscpd.json` in the project works too. See the [jscpd editor docs](https://github.com/kucherenko/jscpd/blob/master/docs/editors.md) for the options.

To turn it off for one language:

```json
{
  "languages": {
    "Rust": { "language_servers": ["!jscpd", "..."] }
  }
}
```

## Languages

Astro, C, C++, C#, Go, Java, JavaScript, Kotlin, PHP, Python, Ruby, Rust, Scala, Svelte, Swift, TSX, TypeScript, Vue.
