Ibis - A federated Wikipedia Alternative
===

A federated Wikipedia alternative. Main objects in terms of federation are the `Instance` and `Article`. Each article belongs to a single origin instance, the one where it was originally created. Articles have a collection of `Edit`s a custom ActivityPub type containing a diff. The text of any article can be built by starting from empty string and applying all associated edits in order. Instances can synchronize their articles with each other, and follow each other to receive updates about articles. Edits are done with diffs which are generated on the backend, and allow for conflict resolution similar to git. Editing also works over federation. In this case an activity `Update/Edit` is sent to the origin instance. If the diff applies cleanly, the origin instance sends the new text in an `Update/Article` activity to its followers. In case there is a conflict, a `Reject` activity is sent back, the editor needs to resolve and resubmit the edit.

## Name

The Ibis is a [bird which is related to the Egyptian god of knowledge and science](https://en.wikipedia.org/wiki/African_sacred_ibis#In_myth_and_legend).

## Development

You need to install [cargo](https://rustup.rs/) and [trunk](https://trunkrs.dev). Then run the following commands in separate terminals:
```
# start backend
cargo run

# start frontend with automatic rebuild on changes
trunk serve -w src/frontend/
```

Then open the site at [127.0.0.1:8080](http://127.0.0.1:8080/). Alternatively you can run `./scripts/watch.sh` to rebuild backend and frontend automatically on code changes (requires [cargo-watch](https://crates.io/crates/cargo-watch)).

## License

[AGPL](LICENSE)
