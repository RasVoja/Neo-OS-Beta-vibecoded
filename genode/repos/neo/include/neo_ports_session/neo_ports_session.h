/*
 * Neo ports on Genode: the session interface of the neo_ports server.
 *
 * Fortress gives a program message ports; on Genode one server component
 * holds the ports and every Neo component reaches them through this
 * session. A port has a name (empty for an unnamed one), a fixed number of
 * places, and a queue of four-word messages.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef _INCLUDE__NEO_PORTS_SESSION__NEO_PORTS_SESSION_H_
#define _INCLUDE__NEO_PORTS_SESSION__NEO_PORTS_SESSION_H_

#include <session/session.h>
#include <base/rpc.h>
#include <util/string.h>

namespace Neo_ports {
	using Name = Genode::String<64>;
	struct Msg { Genode::uint64_t tag, a0, a1, sender; };
	struct Received { bool got; Msg msg; };
	struct Session;
}

struct Neo_ports::Session : Genode::Session
{
	static const char *service_name() { return "Neo_ports"; }
	enum { CAP_QUOTA = 4 };

	/* Port id for name, 0 if there is none; with create, makes it. An
	 * empty name always makes a new unnamed port. */
	virtual unsigned long lookup(Name const &name, bool create, unsigned capacity) = 0;
	/* 0 = queued, 1 = port full, -1 = no such port */
	virtual int send(unsigned long port, Msg msg) = 0;
	virtual Received recv(unsigned long port) = 0;

	GENODE_RPC(Rpc_lookup, unsigned long, lookup, Name const &, bool, unsigned);
	GENODE_RPC(Rpc_send, int, send, unsigned long, Msg);
	GENODE_RPC(Rpc_recv, Received, recv, unsigned long);
	GENODE_RPC_INTERFACE(Rpc_lookup, Rpc_send, Rpc_recv);
};

#endif
