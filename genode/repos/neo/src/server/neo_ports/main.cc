/*
 * neo_ports: holds the message ports of every Neo component on Genode.
 *
 * Ports are shared by all sessions, so a port one component makes (by
 * name) is the same port another finds. The sender of a message is the
 * number of the session it came through.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include <base/component.h>
#include <base/heap.h>
#include <base/log.h>
#include <base/rpc_server.h>
#include <root/component.h>
#include <neo_ports_session/neo_ports_session.h>

namespace Neo_ports {
	struct Port;
	struct Registry;
	struct Session_component;
	struct Root;
	struct Main;
	using namespace Genode;
}

struct Neo_ports::Port
{
	enum { MAX_PLACES = 64 };
	Name     name { };
	unsigned capacity = 0;
	Msg      queue[MAX_PLACES] { };
	unsigned head = 0, count = 0;
	bool     used = false;
};

struct Neo_ports::Registry
{
	enum { MAX_PORTS = 64 };
	Port ports[MAX_PORTS] { };

	unsigned long lookup(Name const &name, bool create, unsigned capacity)
	{
		if (name.length() > 1)
			for (unsigned i = 0; i < MAX_PORTS; i++)
				if (ports[i].used && ports[i].name == name)
					return i + 1;
		if (!create)
			return 0;
		for (unsigned i = 0; i < MAX_PORTS; i++) {
			Port &p = ports[i];
			if (p.used) continue;
			p.used = true;
			p.name = name;
			p.capacity = min(max(capacity, 1u), (unsigned)Port::MAX_PLACES);
			p.head = p.count = 0;
			return i + 1;
		}
		warning("neo_ports: no free port for \"", name, "\"");
		return 0;
	}

	Port *port(unsigned long id) {
		return (id >= 1 && id <= MAX_PORTS && ports[id - 1].used) ? &ports[id - 1] : nullptr; }
};

struct Neo_ports::Session_component : Rpc_object<Session>
{
	Registry      &_registry;
	unsigned long  _id;

	Session_component(Registry &registry, unsigned long id) : _registry(registry), _id(id) { }

	unsigned long lookup(Name const &name, bool create, unsigned capacity) override {
		return _registry.lookup(name, create, capacity); }

	int send(unsigned long id, Msg msg) override
	{
		Port *p = _registry.port(id);
		if (!p) return -1;
		if (p->count == p->capacity) return 1;
		msg.sender = _id;
		p->queue[(p->head + p->count) % Port::MAX_PLACES] = msg;
		p->count++;
		return 0;
	}

	Received recv(unsigned long id) override
	{
		Port *p = _registry.port(id);
		if (!p || p->count == 0) return Received { false, { } };
		Msg const m = p->queue[p->head];
		p->head = (p->head + 1) % Port::MAX_PLACES;
		p->count--;
		return Received { true, m };
	}
};

struct Neo_ports::Root : Root_component<Session_component>
{
	Registry      &_registry;
	unsigned long  _next = 1;

	Session_component *_create_session(const char *) override {
		return new (md_alloc()) Session_component(_registry, _next++); }

	Root(Entrypoint &ep, Allocator &alloc, Registry &registry)
	: Root_component<Session_component>(ep, alloc), _registry(registry) { }
};

struct Neo_ports::Main
{
	Env         &_env;
	Sliced_heap  _heap { _env.ram(), _env.rm() };
	Registry     _registry { };
	Root         _root { _env.ep(), _heap, _registry };

	Main(Env &env) : _env(env) { env.parent().announce(env.ep().manage(_root)); }
};

void Component::construct(Genode::Env &env) { static Neo_ports::Main main(env); }
