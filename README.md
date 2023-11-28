fediwiki
===

A federated Wikipedia alternative. Main objects in terms of federation are the `Instance` and `Article`. Each article belongs to a single origin instance, the one where it was originally created. Articles have a collection of `Edit`s a custom ActivityPub type containing a diff. The text of any article can be built by starting from empty string and applying all associated edits in order. Instances can synchronize their articles with each other, and follow each other to receive updates about articles. Edits are done with diffs which are generated on the backend, and allow for conflict resolution similar to git. Editing also works over federation. In this case an activity `Update/Edit` is sent to the origin instance. If the diff applies cleanly, the origin instance sends the new text in an `Update/Article` activity to its followers. In case there is a conflict, a `Reject` activity is sent back, the editor needs to resolve and resubmit the edit.

## License

[AGPL](LICENSE)
