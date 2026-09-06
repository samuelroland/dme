# Run "cargo check" on all feature flags
check-all:
    #!/bin/bash
    cd app/core
    cargo check
    echo Default features do compile !
    cargo check --features math
    echo Feature math works !
    cargo check --features colored-code
    echo Feature colored-code works !
    cargo check --no-default-features
    echo No default features also works !
    cargo check --no-default-features --features security
    echo Security feature works !
