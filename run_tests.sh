#!/bin/bash

set -e

TERM_PURPLE='\033[0;35m'
TERM_BOLD='\033[1m'
TERM_RESET='\033[0m'

section() {
    echo -e "${TERM_PURPLE}${TERM_BOLD}$1${TERM_RESET}"
}

# argument: "feature1,feature2,..."
test_only_features() {
    section "Only Features: $1"
    cargo test --no-default-features --features "$1" --all-targets > /dev/null
}

test_additional_features() {
    section "Additional Features: $1"
    cargo test --features "$1" --all-targets > /dev/null
}

section "All Default Features"
cargo test --all-targets > /dev/null

section "No Default Features"
cargo test --no-default-features > /dev/null

test_only_features "directory-loading"
