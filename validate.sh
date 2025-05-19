#!/bin/bash

set -exuo pipefail

cargo build

# Our system tests should work with only the built artifacts.
for f in ./tests/*.py ; do
    $f
done

# We should not have leaked development platform information into the repo.
if [ ! -z `git grep $USER`] ; then
    fail "Username is present in repo!"
fi
if [ ! -z `git grep $(hostname)`] ; then
    fail "hostname is present in repo!"
fi

# Once system tests are complete, ensure that the remaining tests pass.
./precheck.sh
