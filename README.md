Ibis - A federated Wikipedia Alternative
===

A federated Wikipedia alternative. Main objects in terms of federation are the `Instance` and `Article`. Each article belongs to a single origin instance, the one where it was originally created. Articles have a collection of `Edit`s a custom ActivityPub type containing a diff. The text of any article can be built by starting from empty string and applying all associated edits in order. Instances can synchronize their articles with each other, and follow each other to receive updates about articles. Edits are done with diffs which are generated on the backend, and allow for conflict resolution similar to git. Editing also works over federation. In this case an activity `Update/Edit` is sent to the origin instance. If the diff applies cleanly, the origin instance sends the new text in an `Update/Article` activity to its followers. In case there is a conflict, a `Reject` activity is sent back, the editor needs to resolve and resubmit the edit.

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

You need to install [cargo](https://rustup.rs/), [trunk](https://trunkrs.dev) and [cargo watch](https://github.com/watchexec/cargo-watch). Run `./scripts/watch.sh` which automatically rebuilds the project after changes. Then open the site at [127.0.0.1:8080](http://127.0.0.1:8080/).

By default the frontend runs on port 8080, which can be changed with env var `TRUNK_SERVE_PORT`. The backend port is 8081 and can be changed with `IBIS_BACKEND_PORT`.

## License

[AGPL](LICENSE)
