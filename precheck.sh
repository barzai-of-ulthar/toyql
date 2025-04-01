#!/bin/bash

set -exuo pipefail

cargo check
cargo clippy
cargo test
