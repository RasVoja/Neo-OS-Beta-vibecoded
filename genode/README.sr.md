# Neo na Genode-u (proba)

Odluka 0001 zadržava Tvrđavu kao jezgro Neo Lite-a za sada, a probu Neo
programa na Genode-u i seL4 počinje odmah. Ova fascikla je ta proba.

- `neo-hello/`, `neo-echo-server/`, `neo-echo-client/` su obični neo-rt
  programi, prevedeni kao statičke biblioteke sa `genode` mogućnošću
  neo-rt-a (jedan Cargo workspace, `Cargo.toml` ovde). U njima nema
  Genode koda.
- `repos/neo/` je Genode repozitorijum:
  - `lib/neo_genode` je izvršna podrška koju povezuje svaka Neo
    komponenta. Ostvaruje sistemske pozive neo-rt-a Genode servisima:
    `exit` ide roditelju, `print` u LOG sesiju, `sleep` Timer-u, a pozivi
    za portove (`find_port`, `new_port`, `send`, `try_send`, `recv`,
    `close`) serveru `neo_ports`. Svaki drugi poziv odgovara „nepoznat
    poziv”, kao što Tvrđava radi za poziv koji nema. Početne ručke
    komponenta dobija iz svoje konfiguracije (`<port handle="0"
    name="echo" create="yes"/>`), kao što Tvrđava daje programu ručke
    kad ga pokrene.
  - `server/neo_ports` drži portove svih Neo komponenti, po imenu.
  - `app/neo_hello`, `app/neo_echo_server`, `app/neo_echo_client` su
    komponente programa.
  - `run/neo_hello.run` podiže jedan program; `run/neo_echo.run` podiže
    par za eho, koji razmeni 100 poruka kroz dva porta.

## Pokretanje

1. Nabavite Genode 24.11 i njegov alatni lanac 23.05, pa pripremite
   portove: `tool/ports/prepare_port sel4 grub2`.
2. `tool/create_builddir x86_64`, pa u `build/x86_64/etc/build.conf`
   postavite `KERNEL ?= sel4`, `BOARD ?= pc`, zakomentarišite `-no-kvm`
   ako ga vaš QEMU ne prihvata, i dodajte
   `REPOSITORIES += /putanja/do/NeoOS/genode/repos/neo`.
3. `GENODE_DIR=/putanja/do/genode scripts/test-genode-x86_64.sh`

Očekivani izlaz sadrži:

```
[init -> neo_hello] Hello from Rust on Genode!
[init -> neo_hello] fib(50) = 12586269025
[init -> neo_hello] find_port: not provided (as expected)
[init] child "neo_hello" exited with exit value 0
[init -> neo_echo_client] echo client: 100 of 100 answers right
[init -> neo_echo_server] echo server: answered 100 messages
```

`neo_hello` nema put do `neo_ports`, pa Genode zapiše i da sesija nije
napravljena; baš to proverava njegov `find_port`.

## Sledeće

Čekanje se za sada proverava jednom u milisekundi; zameniće ga signal od
`neo_ports`. Zatim deljena memorija (Genode dataspace-ovi) i slanje ručki
u porukama, a posle toga neodos ljuska.
