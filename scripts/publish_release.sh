#!/bin/sh
set -e

# Creating the new tag
new_tag="$1"
third_semver=$(echo $new_tag | cut -d "." -f 3)

# Goto the upper route
CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
cd "$CWD/../"


# Update crate versions
old_tag=$(grep version Cargo.toml | head -1 | cut -d'"' -f 2)
sed -i "s/version = \"$old_tag\"/version = \"$new_tag\"/g" Cargo.toml
git add Cargo.toml
cargo check
git add Cargo.lock

# The commit
git commit -m"Version $new_tag"
git tag $new_tag

# Push
git push origin $new_tag
git push
