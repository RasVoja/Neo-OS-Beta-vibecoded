# BeOS i Haiku ideje na Neo-u

[English](BEOS-HAIKU.md) · **Srpski** · Plan, 27. novembar 2026.

Šta uzeti iz BeOS-a i Haikua. Haiku kod je pod MIT licencom (potvrditi po komponenti), pa može da se spoji u Neo, koji je GPLv3 ili novija. Stavke **u specifikaciji** već stoje u specifikaciji (poglavlje 8); H9 do H11 su bili predlozi sa ove strane i **vlasnik ih je prihvatio 27. novembra 2026.**

| # | Ideja | Izvor | Stanje | Šta Neo dobija |
| --- | --- | --- | --- | --- |
| H1 | Tracker kao osnova upravljača fajlovima | Haiku Tracker | u specifikaciji | Ormar raste iz njega, uz prostorne prozore |
| H2 | Interface Kit kao osnova toolkita | Haiku | u specifikaciji | Polazna tačka za Neo Kit |
| H3 | Kartice prozora, slaganje i pločanje | Haiku | u specifikaciji | Prevuci karticu na drugi prozor i postanu jedan; prozori zalepljeni ivicom se pomeraju zajedno |
| H4 | Replikanti | BeOS | u specifikaciji | Deo programa (sat, merač) prevučen na sto tamo živi |
| H5 | Atributi i živi upiti | BFS, Tracker | u specifikaciji | Sačuvana pretraga je folder koji se sam osvežava; kolone atributa u Ormaru |
| H6 | Radni prostori, do devet | BeOS | u specifikaciji | Svaki sa svojom rezolucijom i pozadinom |
| H7 | Jedna nit po prozoru | BeAPI | u specifikaciji | Zaglavljen prozor ne blokira ostale ni traku menija |
| H8 | Vektorske ikone (HVIF) | Haiku | u specifikaciji | Male, oštre na svakoj veličini |
| H9 | **Ideje iz Media Kit-a**: graf medijskih čvorova sa malim kašnjenjem | BeOS Media Kit | prihvaćeno | Uzor za Neo zvučni server, koji treba za [MUSIC.sr.md](MUSIC.sr.md): čvorovi za plejere, efekte i MIDI, tačno vreme na nivou semplova |
| H10 | **MIME tipovi koji nose atribute i omiljene programe** | BeOS | prihvaćeno | Hrani Tipove podataka: svaki tip kaže svoj podrazumevani pregledač, ikonu i dodatne kolone |
| H11 | **Pošta kao upiti** | BeOS Mail | prihvaćeno | Golub prikazuje foldere kao sačuvane upite nad atributima poruka, pa se ništa ne kopira |
| H12 | Paketi kao slojevi | Haiku paketi | delimično u specifikaciji | Neo-ovi neopakovani programi u jednom fajlu to već rade; zadržati pogled „šta program instalira” |
| H13 | Terminal i Process Controller | Haiku | u duhu u specifikaciji | Školjka i Domar |

## Ne uzimati

- Sam BFS (Neo koristi Btrfs sa proširenim atributima) i Haiku app_server (Neo-ov kompozitor je sopstven, na Waylandu).

## Redosled

H1 do H3 i H5 stižu uz rad na radnoj površini i Ormaru. O H9 se odlučuje dok se projektuje zvučni server, pre stavke M1 iz [MUSIC.sr.md](MUSIC.sr.md).
