bench:
    CARGO_PROFILE_BENCH_DEBUG=true CARGO_TARGET_DIR=target-flamegraph cargo flamegraph --root --test ui > /dev/null