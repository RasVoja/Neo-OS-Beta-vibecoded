# Neo na Genode-u (proba)

Odluka 0001 zadržava Tvrđavu kao jezgro Neo Lite-a za sada, a probu Neo
programa na Genode-u i seL4 počinje odmah. Ova fascikla je ta proba.

- `neo-hello/` je običan neo-rt program, preveden kao statička biblioteka
  sa `genode` mogućnošću neo-rt-a. U njemu nema Genode koda.
- `repos/neo/` je Genode repozitorijum. Njegova komponenta `neo_hello`
  povezuje Rust biblioteku i ostvaruje sistemske pozive neo-rt-a Genode
  servisima: `exit` ide roditelju, `print` u LOG sesiju, a svaki drugi
  poziv odgovara „nepoznat poziv”, kao što Tvrđava radi za poziv koji nema.
- `repos/neo/run/neo_hello.run` podiže Genode sa tom jednom komponentom i
  čeka da izađe sa kodom 0.

## Pokretanje

1. Nabavite Genode 24.11 i njegov alatni lanac 23.05, pa pripremite
   portove: `tool/ports/prepare_port sel4 grub2`.
2. `tool/create_builddir x86_64`, pa u `build/x86_64/etc/build.conf`
   postavite `KERNEL ?= sel4`, `BOARD ?= pc`, zakomentarišite `-no-kvm`
   ako ga vaš QEMU ne prihvata, i dodajte
   `REPOSITORIES += /putanja/do/NeoOS/genode/repos/neo`.
3. `GENODE_DIR=/putanja/do/genode scripts/test-genode-x86_64.sh`

Očekivani izlaz se završava sa:

```
[init -> neo_hello] Hello from Rust on Genode!
[init -> neo_hello] fib(50) = 12586269025
[init -> neo_hello] find_port: not provided (as expected)
[init -> neo_hello] neo_hello done
[init] child "neo_hello" exited with exit value 0
```

## Sledeće

Portovi, poruke i deljenje memorije preslikavaju se na Genode RPC i
dataspace-ove (vidi tabelu u odluci 0001); svaki preslikani poziv prestaje
da odgovara „nepoznat poziv”. Sledeći program za probu je neodos ljuska.
