Ibis - A federated Wikipedia Alternative
===

A federated Wikipedia alternative. Main objects in terms of federation are the `Instance` and `Article`. Each article belongs to a single origin instance, the one where it was originally created. Articles have a collection of `Edit`s a custom ActivityPub type containing a diff. The text of any article can be built by starting from empty string and applying all associated edits in order. Instances can synchronize their articles with each other, and follow each other to receive updates about articles. Edits are done with diffs which are generated on the backend, and allow for conflict resolution similar to git. Editing also works over federation. In this case an activity `Update/Edit` is sent to the origin instance. If the diff applies cleanly, the origin instance sends the new text in an `Update/Article` activity to its followers. In case there is a conflict, a `Reject` activity is sent back, the editor needs to resolve and resubmit the edit.

## Name

The Ibis is a [bird which is related to the Egyptian god of knowledge and science](https://en.wikipedia.org/wiki/African_sacred_ibis#In_myth_and_legend).

## How to Use

You can start by reading the main page which is rendered from Markdown. In the "History" tab you can see all the changes that were made to create the current article version. Each change consists of a diff, along with information about the author and the change date. This is similar to a git log. You can discover more articles with the "List Articles" menu option on the left side. By default all known articles from different instances are shown (remote articles are indicated by `@domain`). You can interact with remote articles just the same as local articles. It is also possible to list local articles only.

To continue, register a new account and you are logged in immediately without further confirmation. If you are admin of a newly created instance, login to the automatically created admin account instead. Login details are specified in the config file (by default user `ibis` and password also `ibis`).

On a new instance only the default Main Page will be shown. Use "Create Article" to create a new one. You have to enter the title, text and a summary of the edit. Afterwards press the submit button, and you are redirected to the new article. You can also make changes to existing articles with the "Edit" button at the top. For remote articles, there is additionally a "Fork" option under the "Actions" tab. This allows copying a remote article including the full change history to the local instance. It can be useful if the original instance is dead, or if there are disagreements how the article should be written.

To kickstart federation, paste the domain of a remote instance into the search field, eg `https://example.com`. This will fetch the instance data over Activitypub, and also fetch all articles to make them available locally. The search page will show a link to the instance details page. Here you can follow the instance, so that new articles and edits are automatically federated to your local instance. You can also fetch individual articles from remote instances by pasting the URL into the search field.

## Development

You need to install [cargo](https://rustup.rs/) and [trunk](https://trunkrs.dev). Then run the following commands in separate terminals:
```
# start backend
cargo run

# start frontend with automatic rebuild on changes
trunk serve -w src/frontend/
```

Then open the site at [127.0.0.1:8080](http://127.0.0.1:8080/). Alternatively you can run `./scripts/watch.sh` to rebuild backend and frontend automatically on code changes (requires [cargo-watch](https://crates.io/crates/cargo-watch)).

By default the frontend runs on port 8080, which can be changed with env var `TRUNK_SERVE_PORT`. The backend port is 8081 and can be changed with `IBIS_BACKEND_PORT`.

## License

[AGPL](LICENSE)
