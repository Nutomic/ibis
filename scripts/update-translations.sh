#!/bin/sh
set -e

git remote add weblate http://weblate.join-lemmy.org/git/ibis/ibis/ || true

git fetch weblate
git merge weblate master --squash
git commit -m "Update translations"
