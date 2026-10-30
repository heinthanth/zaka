# Zaka (စံကား)

Zaka is a programming language based on a subset of Burmese. `zakac` checks each line,
catches mistakes before the program runs, explains each one in plain words, and compiles
the program for the Zaka VM.

```zaka
// Print the text inside the quotes.
"မင်္ဂလာပါ ကမ္ဘာကြီး" ဟု ပြသပါ။
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

## Building from source

```sh
cargo build --release -p zakac
./target/release/zakac run program.zaka
```

## AI disclosure

Much of the code, tests and docs here were written with the help of different AI models such as Claude Fable 5.1, Claude Opus 5.5, Qwen 3.8 Max, Gemini 3 Pro, etc. I designed the language, wrote the initial code directed the work and reviewed the changes, and
the tests cover each compiler stage and each runnable example in Learn and Docs. Even so,
AI-written code can hold mistakes that look correct. Treat Zaka as an experimental
project, and please report what seems wrong as an issue.

## Credits

Zaka stands on the work of others. Fonts, crates and packages in [CREDITS.md](CREDITS.md). Special thanks to [PhoenixDigitalArt](https://www.facebook.com/PhoenixDigitalArt)
for the PDA10 Burmese Mono font, and for permission to use it on the website.

## License

Zaka is released under the [MIT license](LICENSE). © 2023–2026 Hein Thant Maung Maung.

The fonts ship under the [SIL Open Font License](https://openfontlicense.org), and the
dependencies keep their own licenses.
