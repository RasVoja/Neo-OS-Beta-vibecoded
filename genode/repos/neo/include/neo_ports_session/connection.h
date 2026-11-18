/*
 * Client side of the Neo_ports session.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#ifndef _INCLUDE__NEO_PORTS_SESSION__CONNECTION_H_
#define _INCLUDE__NEO_PORTS_SESSION__CONNECTION_H_

#include <neo_ports_session/neo_ports_session.h>
#include <base/rpc_client.h>
#include <base/connection.h>

namespace Neo_ports { struct Session_client; struct Connection; }

struct Neo_ports::Session_client : Genode::Rpc_client<Session>
{
	Session_client(Genode::Capability<Session> cap) : Genode::Rpc_client<Session>(cap) { }

	unsigned long lookup(Name const &name, bool create, unsigned capacity) override {
		return call<Rpc_lookup>(name, create, capacity); }

	int send(unsigned long port, Msg msg) override {
		return call<Rpc_send>(port, msg); }

	Received recv(unsigned long port) override {
		return call<Rpc_recv>(port); }
};

struct Neo_ports::Connection : Genode::Connection<Session>, Session_client
{
	Connection(Genode::Env &env)
	:
		Genode::Connection<Session>(env, Label(), Ram_quota { 8*1024 }, Args()),
		Session_client(cap())
	{ }
};

#endif
