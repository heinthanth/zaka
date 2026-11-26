---
title: Editor setup
description: Set up VS Code, Cursor, Zed or another editor for Zaka, and the fonts, input methods and terminal for Burmese text.
---

# Editor setup

Each editor gets the same help with Zaka from the language server that `zakac` runs: it
colors the words, shows errors as we write, shows the data kind (type) of a name on hover,
completes names, goes to a declaration, renames a name in the open files and the files
those import, and formats the file with the rules of `zakafmt`. Install `zakac` first, as
[Installation](installation.md) shows, since each editor starts the server from the
`PATH`.

## VS Code and Cursor

Install the Zaka extension from the extensions view of the editor: search for **Zaka**. It
comes from the VS Code Marketplace in VS Code and from Open VSX in Cursor.

Each [release](https://github.com/heinthanth/zaka/releases) also holds the extension as a
`.vsix` file, which installs from the command line:

```sh
code --install-extension zaka-0.1.0.vsix
cursor --install-extension zaka-0.1.0.vsix
```

Open a `.zaka` or `.zk` file, and the extension starts `zakac lsp`. To use a `zakac` that is
not on the `PATH`, name it in the setting `zaka.server.path`, in the user settings or in the
`.vscode/settings.json` of a folder:

```json
{ "zaka.server.path": "/opt/zaka/bin/zakac" }
```

## Zed

The Zed extension lives in the [Zaka repo](https://github.com/heinthanth/zaka), in the
folder `editors/zed`. Clone the repo, run **zed: install dev extension** in Zed, and pick
that folder. Zed leaves the colors of a language server off until a setting asks for them,
so turn them on for Zaka in the Zed settings:

```json
{
  "languages": {
    "Zaka": { "semantic_tokens": "full" }
  }
}
```

## Other editors

An editor with an LSP client runs `zakac lsp` for files that end in `.zaka` or `.zk`. In
Neovim 0.11 or later, for one, these lines go in `init.lua`:

```lua
vim.filetype.add({ extension = { zaka = "zaka", zk = "zaka" } })
vim.lsp.config("zakac", {
  cmd = { "zakac", "lsp" },
  filetypes = { "zaka" },
  root_markers = { "zakac.toml", ".git" },
})
vim.lsp.enable("zakac")
```

[Editors](../../manual/compiler/editors.md) lists each feature of the server and each
setting of the extensions.

## Burmese text

Zaka reads Burmese text in Unicode. Text written with a Zawgyi font or input method is a
different encoding, and the compiler does not read it as Zaka words, so the editor, the
font and the input method all need to use Unicode.

- **Fonts.** macOS and Windows come with Unicode Burmese fonts (Myanmar Sangam MN on macOS,
  Myanmar Text on Windows), and an editor falls back to them for Burmese letters. On Linux,
  install the Noto Sans Myanmar font from the package of the distribution, such as
  `fonts-noto-core` on Debian and Ubuntu.
- **Input.** Add a Burmese input method to the OS. On macOS, open **System Settings >
  Keyboard > Text Input**, choose **Edit**, and add **Myanmar** or **Myanmar QWERTY**. On
  Windows, add the **Burmese** language under **Settings > Time & language > Language &
  region**. On Linux, add the **Burmese** input source in the settings of the desktop, or
  use [Keyman](https://keyman.com/) with a Unicode Burmese layout.

The names of Zaka programs can be English as well, and the [Names](../variables/names.md)
page tells which letters a name can use.

## Terminal

A program prints Burmese text to the terminal, and most terminals draw the letters of a
Burmese word one at a time, so stacked consonants and vowel signs show apart.
[Ghostty Myanmar](https://ghostty.myanmars.app/), a build of the Ghostty terminal by
Saturngod, draws Burmese words as written, on macOS and Linux. Its source is on
[GitHub](https://github.com/saturngod/ghostty).

Next: [Quick start](quick-start.md).
