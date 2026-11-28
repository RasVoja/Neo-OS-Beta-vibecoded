# Ugrađeni mrežni servisi, drajveri i podaci

[English](NETWORK-DATA.md) · **Srpski** · Plan, 28. novembar 2026.

NBPOS se prema mreži, tipovima fajlova i uređajima ponaša kao AmigaOS: sistem ih poznaje, pa ih svaki program dobija besplatno. Ova strana navodi ugrađene mrežne servise, način na koji se drajveri podižu i šta NBPOS nudi za kućne mreže i baze podataka, uz pošten komentar koliko koja stvar koristi korisniku.

Sve ovde je **[lite]** (ili **[gotovo]** gde tako piše) i radi na običnim x86_64 i ARM računarima. Ništa ne traži Neo hardver. Prve verzije mrežnih servisa rade u **Linux boxu** (domen Aplikacije), a kasnije prelaze u izvorne Neo komponente; licence se ponovo proveravaju pre spajanja koda, a programi pod GPL-2.0-only, AGPL ili drugim licencama isporučuju se kao zasebni programi, nikad spojeni sa kodom NBPOS-a (koji je GPLv3 ili novija).

Oznake: **[gotovo]** radi i testirano je u CI-ju · **[u izradi]** pravi se · **[lite]** planirano za Lite.

## 1. Ideja: tipovi podataka, i za mrežu

Na Amigi (AmigaOS 3.0, 1992) je *datatype* (tip podataka) jednom naučio ceo sistem jedan format: instaliraš JPEG datatype i svaki program koji prikazuje slike otvara JPEG. NBPOS zadržava to pravilo (Tipovi podataka, preko 1.000 formata, dekoderi u pesku) i širi ga na mrežu: veb strana, udaljeni folder, Flash animacija ili Java program su još jedan tip koji sistem poznaje. Program ne nosi svoj pregledač, svoj FTP kod ni svoj prevodilac; pita sistem.

**Šta korisnik dobija:** jedno mesto za ažuriranje i jedan skup pravila privatnosti (Brava pokazuje koji servis dira mrežu); manje programe; isti meni na desni klik svuda.

## 2. Ugrađeni mrežni servisi [lite]

| Servis | Šta korisnik radi | Na osnovu (licenca) | Koliko koristi |
| --- | --- | --- | --- |
| **Prikaz veba kao sistemska usluga** | Pomoć, pošta, Listaj, pregledi u Ormaru i svaki program prikazuju veb strane kroz jednu zajedničku mašinu; Vidik koristi istu | WebKit (WPE WebKit; LGPL-2.1 i BSD) | Mnogo: jedna mašina koju treba držati bezbednom, programi ostaju mali |
| **Udaljeni folderi kao ikone na stolu** | U Ormar upišeš `ftp://`, `sftp://`, `smb://` ili `https://…/webdav`, ili izabereš *Poveži se sa serverom…*; server se pojavi kao ikona diska na stolu, kao disk na Amigi (`NAS:`, `POSAO:`), i otvara se u Ormaru, Neo DOS-u i svakom programu | Neo rukovaoci montiranjem; prvo u Linux boxu sa rclone (MIT) i Samba klijentskim bibliotekama (GPLv3) | Mnogo: nema posebnog FTP ili NAS programa za učenje |
| **Lokalni WebDAV server (po izboru)** | Jedan prekidač u Podešavanjima deli izabrani folder sa drugim računarima i telefonima u kući; podrazumevano isključen, sa lozinkom, samo na kućnoj mreži | mali WebDAV server (npr. rclone serve webdav, MIT) | Srednje: lako deljenje bez oblaka |
| **Flash sadržaj** | Stare `.swf` igre i animacije otvaraju se duplim klikom, u pesku, bez mreže osim ako je dozvoliš | Ruffle (MIT ili Apache-2.0) | Srednje, za ljubitelje retra i stare školske materijale |
| **Java programi** | Dupli klik na `.jar` i program radi; Java okruženje se instalira tek prvi put kad zatreba | OpenJDK (GPL-2.0 sa Classpath izuzetkom), zaseban program | Srednje: stari alati, igre i školski programi i dalje rade |
| **Otvoreni prevod** | Desni klik na dokument, stranu ili označen tekst, *Prevedi na…* | LibreTranslate (AGPL-3.0) sa modelima Argos Translate (MIT), lokalno i bez mreže; vidi [FEATURES.sr.md](FEATURES.sr.md) | Mnogo, za višejezičnu kuću; ništa ne ide u Google Translate ni u drugi oblak |

Svaki servis je van mreže dok ga program ne zatraži, a Brava navodi svaki sa njegovim odlaznim saobraćajem. Servis koji se ne koristi ne pokreće se.

## 3. Samo drajveri koji trebaju (mount i devs)

AmigaOS je držao drajvere uređaja u `DEVS:` i pokretao uređaj tek kad se *montira*, iz njegovog unosa za montiranje. Za hardver kojeg nema ništa se nije učitavalo. NBPOS prati isto pravilo:

- **[gotovo] delimično:** jezgro Tvrđava već pokreće drajver tek kad nađe njegov uređaj: USB (xHCI) samo ako na PCI magistrali postoji USB kontroler, virtio ulaz samo za virtio ulazne uređaje, disk tek kad se disk javi. Dnevnik pokretanja (`BOOTLOG` u Neo DOS-u) pokazuje šta je nađeno i šta je pokrenuto.
- **[lite]:** drajveri žive kao zasebni fajlovi u `DEVS:` (po jedan mali opis za svaki drajver: PCI ili USB klasa, brojevi proizvođača i uređaja). Pri pokretanju sistem pročita PCI, USB i ACPI spiskove i pokrene samo drajvere koji se poklapaju, svaki kao zaseban izolovan program. Uređaj utaknut kasnije (USB, Thunderbolt, mrežni folder) pokreće se u tom trenutku i zaustavlja kad se izvadi.
- **[lite]:** svaki disk se montira namerno i ima ime: `DH0:`, `USB0:`, `NAS:`, `RAD:`. Korisnik vidi i može da promeni unos za montiranje kao običan tekst (Neo DOS `MOUNT`, `ASSIGN` i strana *Uređaji* u Podešavanjima), kao na Amigi.

**Šta korisnik dobija:** brže pokretanje (cilj je manje od 5 sekundi do stola sa SSD-a, vidi listu u [UNIQUE-FEATURES.sr.md](UNIQUE-FEATURES.sr.md)), manje zauzete memorije, manju površinu za napad i jasan odgovor na pitanje „šta radi na mom računaru?”. Pokvaren drajver za jedan uređaj ne može da zaustavi ostale.

## 4. Kućne mreže i telefoni [lite]

| Šta | Upotreba | Na osnovu (licenca) |
| --- | --- | --- |
| **SMB i NFS** | Deljeni folderi na kućnoj mreži i na NAS-u pojavljuju se kao ikone na stolu (odeljak 2); štampači i skeneri deljeni sa drugog računara | Samba klijentske biblioteke (GPLv3), NFS klijent |
| **Syncthing** | Folderi isti na dva ili više računara i telefona, direktno, bez oblaka i bez naloga | Syncthing (MPL-2.0) |
| **Veza sa telefonom** | Fajlovi, ostava, obaveštenja i kamera telefona na računaru; slanje fajla telefonu u blizini | KDE Connect (GPL-2.0 ili novija), LocalSend (Apache-2.0); vidi [demo veze sa telefonom](html/sr/demo/neo-veza.html) |
| **Komšiluk** | Slanje fajlova i pošta između NBPOS računara u kući, pozajmica snage (Zadruga) | vidi [FEATURES.sr.md](FEATURES.sr.md) |

## 5. Baze podataka [lite]

| Šta | Za koga | Na osnovu (licenca) |
| --- | --- | --- |
| **SQLite, ugrađen** | Svaki program: Ormar Pro (pravila Magelana, oznake), Pero (skorašnji fajlovi, šabloni), Rokovnik, Album, Traži, biblioteke Gramofona. Programi dele jednu proverenu biblioteku, korisnik je nikad ne vidi | SQLite (javno dobro) |
| **Mali program za baze** (kasnije) | Kućni korisnici: kolekcija, knjiga recepata, članovi kluba; tabele i obrasci bez pisanja SQL-a, sačuvano kao jedan SQLite fajl | sopstveni program na SQLite-u |
| **PostgreSQL i MariaDB, opcioni serverski paket** | Napredni korisnici i programeri: instalira se iz Riznice samo na zahtev, pokreće se samo kad se koristi, vidljiv u Bravi | PostgreSQL (PostgreSQL licenca), MariaDB (GPL-2.0, zaseban program) |

**Koliko koristi.** SQLite koristi svima, nevidljivo: programi pamte stvari brzo i bezbedno, a pad programa ne kvari podatke. Program za baze koristi porodicama, školama i klubovima koji vode spiskove. Serverski paket je važan samo maloj grupi (programeri, male kancelarije), pa se nikad ne instalira podrazumevano.

## 6. Redosled rada

1. Izvorna mreža (prvo Ethernet, pa Wi-Fi) u Tvrđavi, kao izolovan drajver; DHCP, DNS i TLS u jednom mrežnom servisu.
2. Opisi u `DEVS:` i pokretanje drajvera po poklapanju PCI i USB podataka (nastavak onoga što jezgro već radi).
3. Rukovaoci montiranjem za SMB, SFTP, FTP i WebDAV kroz Linux box; ikone diskova na stolu.
4. SQLite kao zajednička biblioteka za Neo programe.
5. Servis za prikaz veba (WPE WebKit) u Linux boxu; zatim Ruffle, OpenJDK i LibreTranslate kao servisi koji se instaliraju.
6. Syncthing, KDE Connect i LocalSend kao Flatpak programi sa Neo imenima; zatim lokalni WebDAV prekidač i serverski paket.

Povezane strane: [FEATURES.sr.md](FEATURES.sr.md) · [UNIQUE-FEATURES.sr.md](UNIQUE-FEATURES.sr.md) · [AMIGA.sr.md](AMIGA.sr.md) · [BEOS-HAIKU.sr.md](BEOS-HAIKU.sr.md) · [RISC-OS.sr.md](RISC-OS.sr.md) · [CLASSIC-MAC.sr.md](CLASSIC-MAC.sr.md)
