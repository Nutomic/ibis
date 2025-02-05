
![GitHub tag (latest SemVer)](https://img.shields.io/github/tag/Nutomic/ibis.svg)
[![Build Status](https://woodpecker.join-lemmy.org/api/badges/Nutomic/ibis/status.svg)](https://woodpecker.join-lemmy.org/Nutomic/ibis)
[![License](https://img.shields.io/github/license/Nutomic/ibis.svg)](LICENSE)

About Ibis
===

![](assets/logo.png)

Ibis is a federated encyclopedia which uses the ActivityPub protocol, just like Mastodon or Lemmy. Users can read and edit articles seamlessly across different instances. Federation ensures that articles get mirrored across many servers, and
can be read even if the original instance goes down. The software is written in Rust and uses the cutting-edge [Leptos](https://leptos.dev/) framework based on Webassembly. Ibis is fully open source under the AGPL license, to make future enshittification impossible.

Do you want to start a wiki for a TV series, a videogame, about politics, religion or scientific research? Then Ibis is for you! [Setup an instance](https://ibis.wiki/article/Setup_Instructions@ibis.wiki) on your server, it only requires a single binary with PostgreSQL and Nginx. Then you can start editing on the topic of your choice, and connect to other Ibis instances for different topics. Within your own instance you are king, and can decide which articles, which edits and which federation connections are allowed. For more details read the [Usage Instructions](https://ibis.wiki/article/Usage_Instructions@ibis.wiki).

Contributions are more than welcome, especially for the frontend.

## Community

Discuss with other Ibis users on Matrix or Lemmy:

- [Matrix](https://matrix.to/#/#ibis:matrix.org)
- [Lemmy](https://lemmy.ml/c/ibis)

## Useful links

- [Usage Instructions](https://ibis.wiki/article/Usage_Instructions)
- [Setup Instructions](https://ibis.wiki/article/Setup_Instructions)

## Name

The Ibis is a [bird which is related to the Egyptian god of knowledge and science](https://en.wikipedia.org/wiki/African_sacred_ibis#In_myth_and_legend).

## Development

First install PostgreSQL and setup the development database:
```sh
psql -c "CREATE USER ibis WITH PASSWORD 'ibis' SUPERUSER;" -U postgres
psql -c "CREATE DATABASE ibis WITH OWNER ibis;" -U postgres
```

You need to install [cargo](https://rustup.rs/), [pnpm](https://pnpm.io/) and [cargo-leptos](https://github.com/leptos-rs/cargo-leptos). Use `pnpm install` to get Javascript dependencies. You need to enable the wasm target for Rust using `rustup target add wasm32-unknown-unknown`. Then run `cargo leptos watch` which automatically rebuilds the project after changes. Open the site at [localhost:3000](http://localhost:3000/). You can login with user `ibis` and password `ibis`.

The IP and port where the server serves the content can be changed with the env var `LEPTOS_SITE_ADDR`. Defaults to `127.0.0.1:3000`.

## Federation

Main objects in terms of federation are the `Instance` and `Article`. Each article belongs to a single origin instance, the one where it was originally created. Articles have a collection of `Edit`s a custom ActivityPub type containing a diff. The text of any article can be built by starting from empty string and applying all associated edits in order. Instances can synchronize their articles with each other, and follow each other to receive updates about articles. Edits are done with diffs which are generated on the backend, and allow for conflict resolution similar to git. Editing also works over federation. In this case an activity `Update/Edit` is sent to the origin instance. If the diff applies cleanly, the origin instance sends the new text in an `Update/Article` activity to its followers. In case there is a conflict, a `Reject` activity is sent back, the editor needs to resolve and resubmit the edit.

## Donate

Developing a project like this takes a significant amount of work. You can help funding it with donations:

- [Liberapay](https://liberapay.com/Ibis/)
- Bitcoin: `bc1q6mqlqc84q2h55jkkjvex4kc6h9h534rj87rv2l`
- Monero: `84xnACZv82UNTEGNkttLTH8sCeV9Cdr8dHMJSNP6V2hEJW7C17S9xQTUCghwG8TePrRD9wfiPRWcwYvSTHUNoyJ4AXnQYLD`

## License

[AGPL](LICENSE)
