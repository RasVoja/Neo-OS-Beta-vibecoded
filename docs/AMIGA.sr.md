# AmigaOS mogućnosti na Neo-u

[English](AMIGA.md) · **Srpski** · Plan, 18. novembar 2026.

Neo preuzima ideje iz AmigaOS-a: brzu ljusku sa `LoadDesk`, radnu površinu sa fiokama i ikonama, način rada sa datotekama kao u Directory Opus-u i mogućnost da se pokreću klasični Amiga programi. Ova strana navodi otvorene projekte koje planiramo da koristimo, koliko koji košta i šta nam njegova licenca dozvoljava.

## Pravilo za licence

Neo je pod GPLv3 ili novijom. Projekti pod AROS Public License (APL 1.1, zasnovana na MPL 1.1) ili pod samo GPL-2.0 ne mogu se spojiti sa Neo-ovim kodom. Ipak mogu da idu uz Neo **kao zasebni programi**, jer program koji radi na sistemu nije spojen sa njim. Zato svaki od njih držimo kao poseban program, u svojoj fascikli, pod svojom licencom. Kod pod GPL-2.0 ili novijom i pod BSD licencom može se spojiti direktno.

## Stavke

| # | Stavka | Licenca (provereno 18. 11. 2026) | Šta Neo dobija | Procena |
| --- | --- | --- | --- | --- |
| A1 | **Sloj AmigaOS API-ja** iz AROS-a (biblioteke exec, dos, intuition, graphics, icon, workbench) koji radi na Neo-u, kao što AROS radi na Linux-u | APL 1.1, zaseban program | Amiga programi prevedeni iz izvornog koda rade na Neo-u direktno. Na tome počivaju A3 do A7 i to je glavni trošak | 60 č-m |
| A2 | **AROS ljuska i C: naredbe** iza Neo DOS-a | APL 1.1, zasebni programi | Prave AmigaDOS naredbe (`List`, `Copy`, `Assign`, skripte) pored Neo DOS-ovih; Neo DOS 0.3 već ima uređivanje reda i istoriju | 8 č-m |
| A3 | **MUI programi, preko Zune-a** | Zune: APL 1.1 (deo AROS-a). Sam MUI: zatvoren freeware | Zune ostvaruje MUI interfejs, pa MUI programi prevedeni iz izvornog koda rade na Neo-u direktno. Originalni 68k MUI programi rade sa pravim MUI-jem unutar FS-UAE (A8). Za isporuku samog MUI-ja direktno na Neo-u treba licenca od autora; tražićemo je | 12 č-m |
| A4 | **AROS Wanderer** kao izborni „klasični sto” | APL 1.1, zaseban program | Fioke, ikone i pozadina kao u Workbench-u, pokreće se sa `LoadWB`; Neo sto ostaje podrazumevan | 10 č-m |
| A5 | **Directory Opus 5** („dopus5allamigas”, aktivno održavan 2026.) | APL 1.1, zaseban program | Originalni upravljač datotekama za one koji ga žele; Ormar Pro, Neo-ov upravljač, i dalje uzima njegove ideje | 10 č-m |
| A6 | **Galileo** | GPL-2.0 ogranak DOpus-a 5 (repozitorijum ima i tekst GPLv3) | Samo procena: preklapa se sa A5. Uzimamo ispravke i ideje, ne radimo drugi port | 2 č-m |
| A7 | **Scalos** | BSD / GPL-2.0 / LGPL-2.0 (izvori se razlikuju, utvrđuje se po fajlu); neaktivan od 2023. | Drugi klasični sto pored Wanderer-a, sa Scalos-ovim radom sa ikonama i dodacima, prenet na A1 | 8 č-m |
| A8 | **FS-UAE** u Arkadi | GPL-2.0 ili novija, može se spojiti | Pokreće neizmenjene klasične Amiga programe i igre (treba Kickstart ROM korisnika ili AROS ROM) | 6 č-m |
| A9 | **Janus-UAE**, povezani režim | GPL-2.0, zaseban program | Amiga prozori se pojavljuju kao obični prozori na Neo stolu, a ne unutar jednog ekrana emulatora | 10 č-m |

**Ukupno:** oko 126 č-m, oko 20 000 sati. Oko 20 č-m toga se preklapa sa poslom koji specifikacija već broji (Neo DOS 8j, Ormar Pro 8k, Arkada), pa je neto dodatak oko **106 č-m**, oko 2% povrh procene od 5 350 č-m.

## Nasleđe Amige u Neo-ovom sopstvenom kodu

Ne treba svakoj Amiga ideji AROS. Neke idu pravo u NBPOS-ov sopstveni sto.

| # | Stavka | Licenca | Šta Neo dobija | Procena |
| --- | --- | --- | --- | --- |
| A10 | **Povuci ekran** [lite], dodato 28. novembra 2026. | Neo-ov sopstveni kod, GPLv3 ili novija | Povučeš nadole naslovnu traku programa preko celog ekrana i ispod se vidi radni sto ili program iza njega; pustiš, i ekran ostane tu, kao na Amigi. Dugme za dubinu, Super+M za sledeći ekran, tri prsta nadole na dodir. Uz to, po želji, klizanje kao u Windows-u: prozori klize u Traku kad se umanje i izlaze iz nje kad se otvore. U Pro režimu (napredna podešavanja) vidžeti mogu da žive i na sopstvenom ekranu iza programa, do kog se stiže istim pokretom; vidi [FEATURES.sr.md](FEATURES.sr.md). Demo: [povuci ekran](html/sr/demo/neo-povuci-ekran.html) | 2 č-m, računato uz kompozitor |

**Kako to NBPOS radi.** Amiga je imala prave slojevite ekrane: svaki program je mogao da otvori ekran sa sopstvenom rezolucijom i bojama, a video čip je menjao mod red po red, pa su se dva ekrana videla odjednom. Današnja grafička kartica šalje jednu sliku, pa NBPOS nema slojevite video modove. Program preko celog ekrana je jedna površina u kompozitoru. Kad povučeš njegovu naslovnu traku, kompozitor samo pomera tu površinu i crta ono što je ispod (radni sto ili sledeći program); program ništa ne crta ponovo. Program može da zadrži svoju rezoluciju, na primer 640 × 400 za stari softver, a kompozitor ga skalira, pa i dalje izgleda kao njegov sopstveni ekran. Efekat je isti, bez hardverskog trika. Dolazi posle koraka kompozitora „bafer po prozoru” i ne treba mu Neo hardver: dovoljni su običan PC ili Raspberry Pi 5. Video stiže kad funkcija bude u kodu.

## Redosled

1. **Uz M2:** A8 FS-UAE u Arkadi i A2 naredbe ljuske. Jeftine su i vidljive.
2. **Posle M3:** A1 sloj API-ja, pa na njemu A3 Zune, A4 Wanderer i A5 Directory Opus.
3. **Zatim:** A7 Scalos i A9 povezani režim Janus-UAE; A6 Galileo ostaje procena.

Sloj API-ja čeka da Neo pređe na Genode i seL4 (odluka 0001), da bi se pisao jednom, za konačno jezgro.

## Izvori

- Directory Opus 5, APL 1.1: [THIRD_PARTY_NOTICES](https://raw.githubusercontent.com/BlitterStudio/dopus5/master/THIRD_PARTY_NOTICES.md), [Generation Amiga, april 2026.](https://www.generationamiga.com/2026/04/22/the-return-of-directory-opus-amigas-legendary-file-manager-gets-new-life/)
- Galileo: [github.com/Hagbard-Celin/galileo](https://github.com/Hagbard-Celin/galileo)
- Scalos: [Wikipedia](https://en.wikipedia.org/wiki/Scalos), [SourceForge](https://sourceforge.net/projects/scalos/)
- Janus-UAE: [SourceForge](https://sourceforge.net/projects/janus-uae/)
- FS-UAE: [fs-uae.net](https://next.fs-uae.net/download/source)
