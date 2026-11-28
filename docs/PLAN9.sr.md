# Plan 9: omaž i ideje vredne preuzimanja

[English](PLAN9.md) · **Srpski** · Plan, 28. novembar 2026.

**Plan 9 from Bell Labs** je istraživački sistem koji su autori Unix-a (Rob Pajk, Ken Tompson, Dejv Presoto, Fil Vinterbotom i drugi) gradili u Bell Labs-u od kraja osamdesetih. Ime je dobio po filmu Eda Vuda **Plan 9 from Outer Space** (1957, prikazan 1959), koji često zovu najgorim filmom ikad snimljenim, a maskota mu je zeka Glenda. Malo ko ga je koristio, ali mnogo toga u današnjim računarima tiho potiče iz njega: **UTF-8** (Pajk i Tompson, 1992), fajl protokol **9P** koji danas koriste Linux, QEMU i WSL, `/proc` i ideja da mrežni servisi mogu prosto biti fajlovi. Od 2021. kod je pod MIT licencom (Plan 9 Foundation), a fork 9front se i dalje razvija.

NBPOS uzima najbolje iz njega na dva načina: omaž sistemu i filmu, i kratka lista ideja koje Neo čine bezbednijim i jednostavnijim. Sve je **[lite]**: samo softver, Neo hardver nije potreban.

## Omaž [lite]

**Sistemu**
- U Tvrđava kernelu i Neo sistemu montiranja važi pravilo Plana 9: **sve je fajl**, tačnije, svaki resurs koji program koristi ima ime u njegovom imenskom prostoru, a program do njega stiže samo preko porta koji je dobio (odeljak „Ideje” ispod).
- **`p9fs`**: Neo interni mrežni prenos fajlova govori **9P2000** (sa dijalektom `9P2000.L` za Linux kutiju). Svaka strana ove dokumentacije koja pominje deljenje foldera između NBPOS računara, Linux kutije ili virtuelne mašine ispod podrazumeva `p9fs`.
- O programu › Zasluge navodi Plan 9 from Bell Labs pored AmigaOS-a, klasičnog Mac OS-a, BeOS-a i RISC OS-a, sistema od kojih Neo uči.
- UTF-8 je jedino kodiranje teksta u Neo-u; podrška za srpsku latinicu i ćirilicu iz [FEATURES.sr.md](FEATURES.sr.md) stoji na njemu, a zasluge kažu odakle je došao.

**Filmu** (sitno, nikad ne smeta; u Dečjoj sobi isključeno samo ako odrasli tako žele)
- **Plan 9 čuvar ekrana:** leteći tanjiri na vidljivim koncima, ljuljaju se iznad kartonskog groblja, a u uglu Kriswelova rečenica *„Budući događaji poput ovih uticaće na vas u budućnosti”*. Crta ga Neo, bez kadrova iz filma.
- **Neo DOS:** `PLAN9` ispiše jedan red zahvalnosti Bell Labs-u i jedan citat iz filma; `VERSION FULL` pominje oba.
- **Primer u Bioskopu:** sam film se široko navodi kao javno dobro u SAD (autorsko pravo nije obnovljeno). Ako provera licence pre izdanja to potvrdi za zemlje u kojima izlazimo, Bioskop ga može ponuditi kao besplatan klasik za preuzimanje iz Riznice, nikad unapred instaliran.
- **Dijana** zna jednu šalu o njemu, ispriča je najviše jednom godišnje, 22. jula (premijera filma 1959, plus-minus izvor).

## Ideje vredne ugradnje u NBPOS [lite]

| Ideja iz Plana 9 | Šta tamo radi | Neo verzija | Gde pomaže |
| --- | --- | --- | --- |
| **Plumber** | Mali servis dobije deo teksta, ime fajla ili URL i kontekst, a pravila odlučuju koji program to otvara i kako (ime fajla sa `:42` otvara editor na 42. redu) | **Neo Razvodnik** (EN: Neo Plumber), sistemski servis pored Datatypes. Datatypes zna *šta* je fajl; Razvodnik odlučuje *ko* ga otvara i *šta da radi*, i za URL-ove, adrese pošte, brojeve telefona, koordinate na mapi, `IMG_2041.jpg` u redu četa. Pravila su kratak tekstualni fajl i strana u Podešavanjima; svaki program može da mu pošalje (Neo Kit, jedan poziv) | Klik na bilo koju adresu bilo gde i otvara se pravi program; programi iz Linux kutije dobijaju isto preko `xdg-open` |
| **Unija foldera** | `bind -a` ili `bind -b` stavlja više foldera pod jedno ime, pa se ponašaju kao jedan | **Unija montiranja** u Neo sistemu montiranja. Amiga korisnici znaju pola toga: `ASSIGN LIBS: ADD` | Paketi programa vide sistemske fajlove plus svoje; **tema** je jedan folder stavljen ispred podrazumevanog, pa menja samo ono što ima; korisnički fontovi i ikone se slažu preko sistemskih |
| **Imenski prostor po procesu** | Svaki program ima svoj pogled na stablo fajlova; ono čega nema u njemu za njega ne postoji | Tvrđava već daje programima samo portove koje drže; Neo isto tako oblikuje pogled na fajlove. Program vidi `HOME:` samo ako Brava dozvoli, igra vidi svoj folder za snimke, preuzet program ne vidi ništa drugo | Bezbednosna osnova za Bravu, Dečju sobu i Flatpak programe: nema skrivenog pristupa, a Brava može na jednoj strani da pokaže ceo pogled programa |
| **`/net` kao fajlovi** | Mrežne veze su fajlovi u folderu, pa svaki program koji ume da piše fajl može da otvori vezu | Neo mrežni servis pokazuje svoje veze kao stablo samo za čitanje, `NET:`, za Bravu i Neo DOS (`DIR NET:` pokazuje ko s kim priča). Otvaranje veza ostaje poziv porta | Brava-in dnevnik odlaznog saobraćaja i jasan odgovor na „s kim ovaj program priča?” |
| **Komanda `cpu`** | Tvoj terminal, tuđi procesor: udaljena mašina pokreće program, ali vidi tvoje fajlove i ekran kroz tvoj imenski prostor | Funkcija **Zadruga** (pozajmljivanje snage od jačih NBPOS računara) radi ovako, preko `p9fs` | Raspberry Pi 5 šalje FilmCraft render porodičnom PC-ju, bez ručnog kopiranja fajlova |
| **factotum** | Jedan agent drži lozinke i ključeve i obavlja prijave za programe, koji tajnu nikad ne vide | **Ključar** radi kao takav agent za Neo programe | Programi nikad ne čuvaju svoje lozinke |
| **Venti i Fossil** | Stanje diska iz svakog dana se čuva i otvara se kao folder | **Hronos** „nazad na juče” otvara jučerašnji dan kao folder samo za čitanje, `CHRONOS:2026-11-27/` | Vraćanje fajla je obično kopiranje |
| **Acme i rio** | Tekst je interfejs: svaka reč može da se izvrši ili pošalje Razvodniku | Ne izgled (Neo ostaje Mac, Amiga i BeOS po ponašanju), nego jedna navika: u Peru, Neo DOS-u i Ormaru srednji klik na reč šalje je Razvodniku | Navika za iskusne, a ostalima ne smeta |

Šta **ne** uzimamo: izgled rio prozora, navike vezane samo za miša sa tri dugmeta i ideju da korisnik živi u tekst editoru. Neo je za decu, odrasle i profesionalce; ideje Plana 9 idu ispod haube.

## Procena

Oko 6 čovek-meseci povrh planiranog rada: Neo Razvodnik 2, unija montiranja i pogled na imenski prostor 2, `p9fs` (9P2000 i 9P2000.L) 1,5, omaž 0,5. Pogled `NET:`, factotum i folderi Hronosa su delovi Brave, Ključara i Hronosa koji su već u proceni.

## Redosled

1. `p9fs` sa Linux kutijom (najjednostavniji prenos deljenih foldera za nju; QEMU i Linux već govore 9P).
2. Pogled na imenski prostor po procesu u Bravi, sa unijom montiranja za teme.
3. Neo Razvodnik sa Neo Kit-om.
4. Omaž uz desktop (čuvar ekrana, `PLAN9`).

Vidi i [FEATURES.sr.md](FEATURES.sr.md), [AMIGA.sr.md](AMIGA.sr.md), [UNIQUE-FEATURES.sr.md](UNIQUE-FEATURES.sr.md).
