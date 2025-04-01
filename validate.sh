#!/bin/bash

set -exuo pipefail

cargo build

# Our system tests should work with only the built artifacts.
for f in ./tests/*.py ; do
    $f
done

# Once system tests are complete, ensure that the remaining tests pass.
./precheck.sh
