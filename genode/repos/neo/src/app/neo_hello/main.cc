/*
 * neo_hello: runs an unchanged neo-rt program as a Genode component.
 *
 * neo-rt, built with its `genode` feature, sends every system call to
 * neo_genode_call() below. This is the first slice of the mapping in
 * docs/decisions/0001: exit goes to the parent, print goes to the LOG
 * session; every other call answers "unknown call", as Fortress does.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#include <base/component.h>
#include <base/log.h>
#include <base/sleep.h>
#include <util/string.h>

static Genode::Env *_env;

enum : unsigned long long { ERR_UNKNOWN_CALL = ~0ULL - 0xFF };

/* print() may arrive in pieces; Genode logs whole lines. */
static char     _line[256];
static unsigned _line_len;

static void _flush()
{
	_line[_line_len] = 0;
	Genode::log(Genode::Cstring(_line));
	_line_len = 0;
}

static void _put(char c)
{
	if (c == '\n') { _flush(); return; }
	if (_line_len == sizeof(_line) - 1) _flush();
	_line[_line_len++] = c;
}

extern "C" unsigned long long
neo_genode_call(unsigned long long n, unsigned long long a0,
                unsigned long long a1, unsigned long long, unsigned long long)
{
	switch (n) {
	case 0: /* exit */
		if (_line_len) _flush();
		_env->parent().exit((int)(long long)a0);
		Genode::sleep_forever();
	case 1: /* print */
		{
			char const *s = (char const *)a0;
			for (unsigned long long i = 0; i < a1; i++) _put(s[i]);
			return a1;
		}
	default:
		return ERR_UNKNOWN_CALL;
	}
}

extern "C" long long neo_genode_main();

void Component::construct(Genode::Env &env)
{
	_env = &env;
	long long const r = neo_genode_main();
	if (_line_len) _flush();
	env.parent().exit((int)r);
}
