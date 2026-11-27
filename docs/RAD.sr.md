# RAD: u osnovnoj verziji

[English](RAD.md) · **Srpski** · Plan, 27. novembar 2026.

`RAD:` je brzi disk u memoriji koji čuva sadržaj preko restarta („Radni sto” u Lako režimu). Vlasnik je odlučio da pripada **osnovnoj Lite verziji**, ne concept artu. Specifikacija ga je opisala za Neo hardver sa pokretačkim ROM-om; ova strana ga deli na nivoe koje softver može da isporuči na običnom PC-ju.

| Nivo | Šta | Traži | Stanje |
| --- | --- | --- | --- |
| 1 | `RAM:` i `RAD:` kao volumeni u memoriji u Neo DOS-u i Ormaru; svaki blok ima kontrolni zbir, oštećen volumen se ne montira i poruka objašnjava zašto | servis za blokove u memoriji, servis fajl sistema | za uraditi |
| 2 | „Sačuvaj RAD: pri gašenju”: sadržaj se upisuje na disk (sažeto) i vraća pri sledećem paljenju | drajver za upis na disk (NVMe ili AHCI), koji još ne postoji | blokirano |
| 3 | **Preživljava restart koji Neo sam pokrene** (restart posle ažuriranja, „Restartuj” u meniju): Fortress čuva rezervisan region, vraća se na svoju ulaznu tačku bez resetovanja memorije od strane firmvera, nalazi region po zaglavlju i proverava ga | rezervisan region iz mape memorije, put restarta koji ponovo inicijalizuje uređaje (xHCI i druge), proba u QEMU-u | za uraditi |
| 4 | Preživljava nestanak struje | ultrakondenzator ili baterija, kao na Neo pločama (konceptni hardver) | **koncept** |

**Šta običan PC ne može da obeća:** hardverski reset ili isključenje struje može da obriše memoriju, a ne mora; odlučuje firmver. Zato se nivo 3 obećava samo za restarte koje Neo sam izvodi. Za sve ostalo važi nivo 2, ili iskrena poruka.

## Podešavanja

- Veličina: deo memorije, podesiv. Predlog je jedna osmina RAM-a podrazumevano (specifikacija je za najmanji model dala 4 GB od 32 GB); memoriju troše samo fajlovi koje drži.
- Oznake Brave važe. Na Lite-u nema Sef čipa, pa se fajlovi sa oznakom Lično ne drže u `RAD:` osim ako je šifrovan lozinkom.
- Podizanje sistema sa njega, kao na Amigi, za brze probe slika.

## Proba

CI proba paljenja koja piše u `RAD:`, restartuje preko Fortress-a i čita sadržaj nazad, u QEMU-u sa BIOS-om i sa UEFI-jem.
