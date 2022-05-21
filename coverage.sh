RUSTFLAGS="-C instrument-coverage" \
RUSTDOCFLAGS="-C instrument-coverage -Z --persist-doctests target/debug/doctestbins" \
LLVM_PROFILE_FILE="image_sieve-%m.profraw" \
    cargo test --tests -j4
$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata merge -sparse image_sieve-*.profraw -o image_sieve.profdata
export TEST_FILES=$(
    for file in $(RUSTFLAGS="-C instrument-coverage" cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM )
    do
        printf "%s %s " -object $file;
    done
)
$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov report $TEST_FILES --instr-profile=image_sieve.profdata --summary-only
