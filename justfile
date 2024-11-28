update_dependencies:
    git submodule update --recursive --remote && cargo clean && cargo update && cargo build
update_submodules:
    git submodule update --recursive --remote
hard_update_submodules:
    git submodule deinit -f libpd && git rm -f libpd && rm -rf .git/modules/libpd && git submodule add -b extended https://github.com/alisomay/libpd.git libpd && git submodule update --init --remote --recursive 
test_publish:
    cd libpd && git stash && cd .. && git submodule update --init --recursive --remote && cargo publish --dry-run --no-verify --allow-dirty
publish:
    cd libpd && git stash && cd .. && git submodule update --init --recursive --remote && cargo publish --no-verify --allow-dirty