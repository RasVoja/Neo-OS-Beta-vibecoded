# The runtime every Neo component links: neo-rt's system calls on Genode.
SRC_CC = neo_genode.cc
LIBS   = base
vpath neo_genode.cc $(REP_DIR)/src/lib/neo_genode
