update_dependencies:
    git submodule update --recursive --remote && cargo clean && cargo update && cargo build
update_submodules:
    git submodule update --recursive --remote