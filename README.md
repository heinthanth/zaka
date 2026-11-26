# Zaka (စံကား)

Zaka is a programming language based on a subset of Burmese. Burmese sentences are compiled down to instructions for the Zaka VM, or to machine code through Cranelift or C.

```zaka
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသမည်။
```

## Quick links

- [Website](https://zaka-lang.org): the home of Zaka
- [Learn](https://zaka-lang.org/learn/): the language, taught through runnable
  examples ([source](docs/learn/))
- [Docs](https://zaka-lang.org/docs/): the full rules, the compiler and the VM
  ([source](docs/manual/))
- [Explorer](https://zaka-lang.org/explorer/): compile and run Zaka in the
  browser, one stage at a time
- [Samples](samples/): example programs with their expected output

## Install

On macOS and Linux:

```sh
curl -fsSL https://zaka-lang.org/install.sh | sh
```

On Windows, in PowerShell:

```powershell
irm https://zaka-lang.org/install.ps1 | iex
```

Each [release](../../releases) also holds `zakac` for macOS (Apple
silicon), Linux and Windows (x86_64 and ARM64 each), and the VS Code extension
(`.vsix`). [Get started](https://zaka-lang.org/learn/get-started/) covers the install,
the editors and a first program.

## Building from source

```sh
cargo build --release -p zakac
./target/release/zakac run program.zaka
```

## AI disclosure

Much of the code, tests and docs here were written with the help of different AI models such as Anthropic Claude, OpenAI GPT, Alibaba Qwen, Google Gemini, etc. I designed the language, wrote the initial code, directed the work and reviewed the changes. And the unit tests, integration tests cover each compiler stage and each runnable example in Learn and Docs. Even so, AI-written code can hold mistakes that look correct. Treat Zaka as an experimental project, and please report what seems wrong as an issue.

## Acknowledgement

I'm grateful to [Robert Nystrom](https://github.com/munificent) for
[*Crafting Interpreters*](https://craftinginterpreters.com/). I came to compiler
development knowing nothing about it, and the book was a companion from the first page:
whenever I was stuck, its clear explanations and complete code showed me the next step.

I'm also grateful to Sayar [Thet Khine](https://github.com/mrthetkhine). When I heard that
he had created UCSY Language, I was so impressed that I wanted to build a language too.
That wish started me on the path that led to Zaka.

## Credits

Zaka stands on the work of others. Fonts, crates and packages in [CREDITS.md](CREDITS.md). Special thanks to [PhoenixDigitalArt](https://www.facebook.com/PhoenixDigitalArt)
for the PDA10 Burmese Mono font, and for permission to use it on the website.

## License

Zaka is released under the [MIT license](LICENSE). © 2023–2026 Hein Thant Maung Maung.<br/>
Most of the fonts are shipped under the [SIL Open Font License](https://openfontlicense.org) and other dependencies are distributed under their own license.
