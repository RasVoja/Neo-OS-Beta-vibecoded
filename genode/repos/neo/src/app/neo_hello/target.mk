# The C++ side of neo_hello; the Rust side is a static library built by
# cargo from genode/neo-hello (see run/neo_hello.run).
TARGET   = neo_hello
SRC_CC   = main.cc
LIBS     = base
NEO_RUST = $(REP_DIR)/../../neo-hello/target/x86_64-unknown-none/release/libneo_hello.a
EXT_OBJECTS += $(NEO_RUST)
$(TARGET): $(NEO_RUST)
