# A Neo program on Genode: the Rust static library built by cargo from
# genode/neo-echo-client, linked with the neo_genode runtime (see run/neo_*.run).
TARGET   = neo_echo_client
SRC_CC   = neo_program.cc
LIBS     = base neo_genode
vpath neo_program.cc $(REP_DIR)/src/lib/neo_genode
NEO_RUST = $(REP_DIR)/../../target/x86_64-unknown-none/release/libneo_echo_client.a
EXT_OBJECTS += $(NEO_RUST)
$(TARGET): $(NEO_RUST)
