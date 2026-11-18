# Odluka 0001: kada Tvrđava ustupa mesto Genode-u na seL4

*English: [0001-fortress-and-genode.md](0001-fortress-and-genode.md)*

**Stanje:** prihvaćeno (2026-11-15), uz jednu izmenu vlasnika projekta: proba na Genode-u (treći uslov ispod) počinje odmah, pre zatvaranja M1, umesto da čeka druga dva uslova.

**Proba:** prvi neo-rt program radi neizmenjen kao Genode komponenta na seL4 pod QEMU-om (`genode/`, 2026-11-18).

## Polazište

README planira NeoOS na okviru Genode iznad mikrojezgra seL4. Tvrđava,
jezgro u ovom repozitorijumu, počela je kao početno jezgro: malo jezgro da
se nauči put do hardvera i da od prvog dana postoji nešto što se može
testirati. Posle četrdeset koraka ima dve arhitekture, drajvere u
korisničkom režimu, sposobnosti, portove za poruke, deljenu memoriju i
prstenove, radnu površinu sa prozorima i Neo DOS, sve pokriveno testovima
podizanja u CI-ju.

Ako predugo ostanemo na Tvrđavi, sami ćemo pisati ono što seL4 već dokazuje
(izolaciju, proverenu ispravnost jezgra) i ono što Genode već ima (stotine
drajvera, grafički sloj, portove Mese i programa nivoa Chromium-a). Ako
pređemo prerano, zaustavićemo vidljiv napredak koji privlači saradnike, a
Genode se ni sam ne uči brzo.

## Mogućnosti

1. **Preći odmah.** Prekinuti rad na Tvrđavi i početi iznova na Genode/seL4.
2. **Nikad ne preći.** Razviti Tvrđavu u puno jezgro.
3. **Preći u jasnom trenutku, zadržavajući korisnički deo.** Korisnički deo
   (programi, `neo-rt`, `neo-gfx`, `neo-keys`, `neo-ring`, servis za ekran,
   Neo DOS) graditi tako da ne zavisi od unutrašnjosti Tvrđave, pa ga
   preneti na Genode kad se ispune uslovi ispod. Tvrđava ostaje jezgro za
   učenje i rezerva „Obnova” (Revive).

## Odluka

Mogućnost 3. Korisnički deo razgovara sa jezgrom samo kroz `neo-rt`, pa je
`neo-rt` jedino mesto koje se menja kad se menja jezgro. Njegovi pozivi se
na Genode preslikavaju ovako:

| `neo-rt` danas | Na Genode-u |
| --- | --- |
| portovi, `send`, `recv`, `try_send` | RPC i signali između komponenti |
| `share`, `send_handle`, `map_device` | dataspace-ovi i prosleđivanje sposobnosti |
| `neo-ring` preko deljenog bloka | dataspace i signal (Genode-ov packet stream je ista ideja) |
| fajlovi, volumeni | VFS i sesije sistema fajlova |
| protokol ekrana, prozori | GUI sesija (nitpicker ili njegov naslednik) |
| tastatura, pokazivač, raspored | ulazne sesije; `neo-keys` zadržava rasporede |
| `spawn`, sposobnosti date pri pokretanju | rutiranje komponente init |

**Uslovi za prelazak.** Prelazak počinje kad važe *sva tri*:

- M1 je zatvoren: kod je javan i saradnici imaju početne zadatke.
- Izabran je prvi pravi hardver (PC sa UEFI-jem ili Raspberry Pi), jer bi
  tu Tvrđavi trebali drajveri koje Genode već ima.
- Jedan Neo program (servis za ekran ili Neo DOS) je probno izgrađen za
  Genode, kroz `neo-rt` pozadinu za Genode, i njegov test podizanja prolazi
  pod QEMU-om.

Do tada se novi rad na jezgru Tvrđave ograničava na ono što treba
korisničkom delu i na održavanje testova zelenim. U Tvrđavi se ne pišu novi
drajveri koje Genode već ima.

## Posledice

- Sati rada na jezgru Tvrđave računaju se u stavke specifikacije za
  Tvrđavu i Exec-N samo delimično, kao što izveštaj o satima već kaže.
- `neo-rt` ne sme da propušta detalje Tvrđave (brojeve handle-ova,
  brojeve poziva) u programe preko onoga što je dokumentovano.
- Testovi podizanja ostaju; pod Genode-om postaju run skripte koje
  proveravaju iste redove.
