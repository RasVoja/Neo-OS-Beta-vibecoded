/*
 * neo_genode: the runtime that lets an unchanged neo-rt program run as a
 * Genode component.
 *
 * neo-rt, built with its `genode` feature, sends every system call to
 * neo_genode_call() below. This is the mapping in docs/decisions/0001,
 * one slice at a time:
 *
 *   exit (0)            the parent
 *   print (1)           the LOG session, a line at a time
 *   find_port (2)       the neo_ports server, by name
 *   send (3), try_send (24), recv (5), close (7), new_port (23)
 *                       the neo_ports server
 *   sleep (4)           the Timer session (a tick is 10 ms, as on Fortress)
 *
 * Every other call answers "unknown call", as Fortress does for a call it
 * lacks. A component's starting handles come from its config, the way
 * Fortress hands a program its capabilities when it starts it:
 *
 *   <config> <port handle="0" name="echo" create="yes" capacity="16"/> </config>
 *
 * The neo_ports and Timer sessions are opened on first use, so a program
 * that never uses them needs neither. Waiting (a blocking send or recv)
 * polls the server once a millisecond: simple, and good enough for the
 * trial; a signal from the server will replace it.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include <base/attached_rom_dataspace.h>
#include <base/component.h>
#include <base/log.h>
#include <base/sleep.h>
#include <neo_ports_session/connection.h>
#include <timer_session/connection.h>
#include <util/reconstructible.h>
#include <util/string.h>

using namespace Genode;

typedef unsigned long long u64;

enum : u64 {
	ERR_BASE         = ~0ULL - 0xFF,
	ERR_UNKNOWN_CALL = ERR_BASE,
	ERR_BAD_POINTER  = ERR_BASE + 1,
	ERR_NOT_FOUND    = ERR_BASE + 2,
	ERR_BAD_HANDLE   = ERR_BASE + 3,
	NO_HANDLE        = ~0ULL,
};

static Env *_env;

/* print() may arrive in pieces; Genode logs whole lines. */
static char     _line[256];
static unsigned _line_len;

static void _flush()
{
	_line[_line_len] = 0;
	log(Cstring(_line));
	_line_len = 0;
}

static void _put(char c)
{
	if (c == '\n') { _flush(); return; }
	if (_line_len == sizeof(_line) - 1) _flush();
	_line[_line_len++] = c;
}

/* Handles: index = neo-rt handle, value = neo_ports port id (0 = free). */
enum { MAX_HANDLES = 32 };
static unsigned long _handles[MAX_HANDLES];

static Constructible<Neo_ports::Connection> _ports;
static bool _ports_denied;

static Neo_ports::Connection *_ports_session()
{
	if (!_ports.constructed() && !_ports_denied) {
		try { _ports.construct(*_env); }
		catch (...) { _ports_denied = true; }
	}
	return _ports.constructed() ? &*_ports : nullptr;
}

static Constructible<Timer::Connection> _timer;

static void _msleep(u64 ms)
{
	if (!_timer.constructed()) _timer.construct(*_env);
	_timer->msleep(ms);
}

static u64 _new_handle(unsigned long port)
{
	for (unsigned h = 0; h < MAX_HANDLES; h++)
		if (!_handles[h]) { _handles[h] = port; return h; }
	return ERR_BAD_HANDLE;
}

static unsigned long _port(u64 h) { return h < MAX_HANDLES ? _handles[h] : 0; }

static u64 _send(u64 h, u64 tag, u64 a0, u64 a1, bool wait)
{
	Neo_ports::Connection *ports = _ports_session();
	if (!ports) return ERR_UNKNOWN_CALL;
	unsigned long const port = _port(h);
	if (!port) return ERR_BAD_HANDLE;
	for (;;) {
		int const r = ports->send(port, Neo_ports::Msg { tag, a0, a1, 0 });
		if (r < 0) return ERR_BAD_HANDLE;
		if (r == 0) return 0;
		if (!wait) return 1; /* try_send: full */
		_msleep(1);
	}
}

static u64 _recv(u64 h, u64 *out, bool poll)
{
	Neo_ports::Connection *ports = _ports_session();
	if (!ports) return ERR_UNKNOWN_CALL;
	unsigned long const port = _port(h);
	if (!port) return ERR_BAD_HANDLE;
	if (!out) return ERR_BAD_POINTER;
	for (;;) {
		Neo_ports::Received const r = ports->recv(port);
		if (r.got) {
			out[0] = r.msg.tag; out[1] = r.msg.a0; out[2] = r.msg.a1;
			out[3] = r.msg.sender; out[4] = NO_HANDLE; out[5] = NO_HANDLE;
			return 0;
		}
		if (poll) return ERR_NOT_FOUND;
		_msleep(1);
	}
}

extern "C" u64 neo_genode_call(u64 n, u64 a0, u64 a1, u64 a2, u64 a3)
{
	switch (n) {
	case 0: /* exit */
		if (_line_len) _flush();
		_env->parent().exit((int)(long long)a0);
		sleep_forever();
	case 1: /* print */
		for (u64 i = 0; i < a1; i++) _put(((char const *)a0)[i]);
		return a1;
	case 2: { /* find_port */
		Neo_ports::Connection *ports = _ports_session();
		if (!ports) return ERR_UNKNOWN_CALL;
		char name[64] { };
		copy_cstring(name, (char const *)a0, min((size_t)a1 + 1, sizeof(name)));
		unsigned long const port = ports->lookup(Neo_ports::Name(Cstring(name)), false, 0);
		return port ? _new_handle(port) : ERR_NOT_FOUND;
	}
	case 3:  return _send(a0, a1, a2, a3, true);
	case 24: return _send(a0, a1, a2, a3, false);
	case 4:  _msleep(a0 * 10); return 0;
	case 5:  return _recv(a0, (u64 *)a1, a2 != 0);
	case 7:
		if (!_port(a0)) return ERR_BAD_HANDLE;
		_handles[a0] = 0;
		return 0;
	case 23: { /* new_port */
		Neo_ports::Connection *ports = _ports_session();
		if (!ports) return ERR_UNKNOWN_CALL;
		unsigned long const port = ports->lookup(Neo_ports::Name(), true, (unsigned)a0);
		return port ? _new_handle(port) : ERR_NOT_FOUND;
	}
	default:
		(void)a3;
		return ERR_UNKNOWN_CALL;
	}
}

extern "C" long long neo_genode_main();

void Component::construct(Env &env)
{
	_env = &env;

	/* Starting handles from the config. */
	try {
		static Attached_rom_dataspace config(env, "config");
		config.xml().for_each_sub_node("port", [&] (Xml_node const &node) {
			unsigned const h = node.attribute_value("handle", 0u);
			auto const name  = node.attribute_value("name", Neo_ports::Name());
			bool const create = node.attribute_value("create", false);
			unsigned const capacity = node.attribute_value("capacity", 16u);
			Neo_ports::Connection *ports = _ports_session();
			if (!ports || h >= MAX_HANDLES) return;
			_handles[h] = ports->lookup(name, create, capacity);
			if (!_handles[h])
				warning("neo: port \"", name, "\" not found for handle ", h);
		});
	} catch (...) { }

	long long const r = neo_genode_main();
	if (_line_len) _flush();
	env.parent().exit((int)r);
}
