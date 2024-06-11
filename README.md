About Ibis
===

Ibis is a federated online encyclopedia similar to Wikipedia.  Users can read, create and edit articles seamlessly across instances. It uses the Activitypub protocol to connect users across different websites, similar to Mastodon or Lemmy. This ensures that there is no single point of control which may be used for global censorship. Instead each Ibis instance is independent and controlled by its admin. Admins can decide which rules to enforce, which content to allow and which instances to connect with. Users who are unhappy with the rules can easily setup their own Ibis instance with their own rules. 

The project uses the same technology as [Lemmy](https://join-lemmy.org/) and benefits from lessons learned during its development. It is currently in a proof of concept stage. Core features are already working, including creation and editing of articles, full federation and a basic frontend. You can see it in action on [ibis.wiki](https://ibis.wiki). However more work is needed to get the project ready for production use, to add features like moderation tools, user account management, media support, article discussions and better web design. Contributions are welcome!

Read the [Project Announcement](https://ibis.wiki/article/Announcing_Ibis,_the_federated_Wikipedia_Alternative) for more information.

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

## Federation

Main objects in terms of federation are the `Instance` and `Article`. Each article belongs to a single origin instance, the one where it was originally created. Articles have a collection of `Edit`s a custom ActivityPub type containing a diff. The text of any article can be built by starting from empty string and applying all associated edits in order. Instances can synchronize their articles with each other, and follow each other to receive updates about articles. Edits are done with diffs which are generated on the backend, and allow for conflict resolution similar to git. Editing also works over federation. In this case an activity `Update/Edit` is sent to the origin instance. If the diff applies cleanly, the origin instance sends the new text in an `Update/Article` activity to its followers. In case there is a conflict, a `Reject` activity is sent back, the editor needs to resolve and resubmit the edit.

## License

[AGPL](LICENSE)
