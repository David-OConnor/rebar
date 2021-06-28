[![crates.io version](https://meritbadge.herokuapp.com/seed)](https://crates.io/crates/seed)
[![crates.io downloads](https://img.shields.io/crates/d/seed.svg)](https://crates.io/crates/seed)
[![docs.rs](https://docs.rs/seed/badge.svg)](https://docs.rs/seed)
[![Built with cargo-make](https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg)](https://sagiegurari.github.io/cargo-make)

<p align="center">
  <img src="https://raw.githubusercontent.com/seed-rs/seed-rs.org/81ed1acc77062ede3295683f21f2d39611843192/seed_branding/seed_logo.min.svg" width="256" title="Seed logo">
</p>

### [Website](https://seed-rs.org) | [Discord](https://discord.gg/JHHcHp5)
---
Seed is a Rust front-end framework for creating fast and reliable web apps with an Elm-like architecture.

- completely written in Rust, including the templating system (e.g. `div!` macro).
- built-in state management that is based on the Elm architecture.
- a batteries-included approach with a focus on developer experience.
- clear and extensive documentation for Rust beginners and pros alike.
- WebAssembly.

## Why Seed?
Seed allows you to develop the front-end with all the benefits of Rust, meaning speed, safety, and too many more things to count.

The Seed templating system uses a macro syntax that makes Rustaceans feel right at home. This means linting, formatting, and commenting will work, and it's all in Rust. This is opposed to a JSX-like syntax that relies on IDE extensions to improve the developer experience.

Seed has a batteries-included approach. This means less time writing boilerplate and less time installing dependencies.

## Why not Seed?
- It's newer. It's harder to find support outside of Discord.
- WebAssembly is newer with less support. [Browser compatibility is at 92.9%](https://caniuse.com/?search=webassembly).
- Pre-built components are rare. You will likely have to roll your own components such as date pickers.
- No server-side rendering yet [(#232)](#232).
- You may prefer other Rust frameworks like [MoonZoon](https://github.com/MoonZoon/MoonZoon) or [Yew](https://github.com/yewstack/yew).

## Getting Started
To get started right away, we can use the quickstart template:
```sh
cargo install cargo-generate
cargo install trunk
cargo install wasm-bindgen-cli
cargo generate --git https://github.com/seed-rs/seed-quickstart.git --name seed-quickstart
cd seed-quickstart
trunk serve
```

If you get an error about wasm being linked against a different version of `wasm-bindgen`, just follow the suggestion to run `cargo update -p wasm-bindgen`. This will fix the linkings.

You should now see a working counter app in your browser at `localhost:8080`.

## Getting into Seed
The [Seed website](https://seed-rs.org/) and the [library docs](https://docs.rs/seed/latest) are the best way to learn about the functionalities of Seed.

The [Seed examples](examples/) are another good resource.

[Trunk](https://github.com/thedodd/trunk) is the recommended application bundler for Seed. Seed projects are typically run with `trunk serve` instead of `cargo run`. You might also see `cargo make start project_name` in the examples. Going forward, we recommend using Trunk. 

[Seed Styles](https://github.com/seed-rs/styles_hooks) is a styling library for Seed to create global and scoped styles.

To use web APIs, there is [web-sys](https://github.com/rustwasm/wasm-bindgen/tree/master/crates/web-sys) which is a part of the [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) project. wasm-bindgen is a dependency of Seed.

There are also two template repositories. However, they are not currently up to date.
- [Quickstart](https://github.com/seed-rs/seed-quickstart)
- [Webpack quickstart](https://github.com/seed-rs/seed-quickstart-webpack)

## FAQ
### How stable is Seed?
As a framework, Seed is mostly feature-complete. You can build complete web apps in Seed. Projects built in Seed do use Rust `stable`. Being in Rust, it's easy to create robust, predictable programs.

### What's next for Seed?
Check out the [roadmap](https://github.com/seed-rs/seed/milestones).

## Documentation
- Guides can be found at [seed-rs.org](https://seed-rs.org)
- API documentation can be found at [docs.rs/seed](https://docs.rs/seed)

## Resources
### Seed
- [Awesome-seed-rs](https://github.com/seed-rs/awesome-seed-rs): A curated list of resources
- [Seed Realworld](https://github.com/seed-rs/seed-rs-realworld): A detailed realworld example (outdated but still helpful)
- [Engineering Rust Web Applications](https://erwabook.com/intro/): A book describing full-stack Rust web development using Seed (oudated but still helpful)

### Rust
- [Rust Discord](https://discordapp.com/invite/rust-lang)
- [Rust IRC](https://www.irccloud.com/invite?channel=%23%23rust&hostname=chat.freenode.net&port=6697&ssl=1)

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md).

## Supported By
See [BACKERS.md](BACKERS.md).

<p>This project is supported by:</p>
<p>
  <!-- referral link from console -->
  <a href="https://m.do.co/c/f02c252209c1">
    <img src="https://opensource.nyc3.cdn.digitaloceanspaces.com/attribution/assets/SVG/DO_Logo_horizontal_blue.svg" width="201px">
  </a>
</p>

The [Seed website](https://seed-rs.org) is served by Netlify.

[![Netlify](https://www.netlify.com/img/global/badges/netlify-light.svg)](https://www.netlify.com)
