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
| A3 | **Zune** umesto MUI-ja | APL 1.1 (deo AROS-a) | Skup alata za prozore kompatibilan sa MUI-jem, pa se MUI programi prevode na Neo-u. Sam MUI je zatvoren freeware, pa ga ne koristimo | 12 č-m |
| A4 | **AROS Wanderer** kao izborni „klasični sto” | APL 1.1, zaseban program | Fioke, ikone i pozadina kao u Workbench-u, pokreće se sa `LoadWB`; Neo sto ostaje podrazumevan | 10 č-m |
| A5 | **Directory Opus 5** („dopus5allamigas”, aktivno održavan 2026.) | APL 1.1, zaseban program | Originalni upravljač datotekama za one koji ga žele; Ormar Pro, Neo-ov upravljač, i dalje uzima njegove ideje | 10 č-m |
| A6 | **Galileo** | GPL-2.0 ogranak DOpus-a 5 (repozitorijum ima i tekst GPLv3) | Samo procena: preklapa se sa A5. Uzimamo ispravke i ideje, ne radimo drugi port | 2 č-m |
| A7 | **Scalos** | BSD / GPL-2.0 / LGPL-2.0 (izvori se razlikuju); neaktivan od 2023. | Ideje za Neo sto (ikone, dodaci); port samo ako ga neki dobrovoljac želi | 8 č-m (izborno) |
| A8 | **FS-UAE** u Arkadi | GPL-2.0 ili novija, može se spojiti | Pokreće neizmenjene klasične Amiga programe i igre (treba Kickstart ROM korisnika ili AROS ROM) | 6 č-m |
| A9 | **Janus-UAE**, povezani režim | GPL-2.0, zaseban program | Amiga prozori se pojavljuju kao obični prozori na Neo stolu, a ne unutar jednog ekrana emulatora | 10 č-m |

**Ukupno:** oko 126 č-m, oko 20 000 sati. Oko 20 č-m toga se preklapa sa poslom koji specifikacija već broji (Neo DOS 8j, Ormar Pro 8k, Arkada), pa je neto dodatak oko **106 č-m**, oko 2% povrh procene od 5 350 č-m.

## Redosled

1. **Uz M2:** A8 FS-UAE u Arkadi i A2 naredbe ljuske. Jeftine su i vidljive.
2. **Posle M3:** A1 sloj API-ja, pa na njemu A3 Zune, A4 Wanderer i A5 Directory Opus.
3. **Kasnije ili od dobrovoljaca:** A9 povezani režim Janus-UAE, A6 i A7.

Sloj API-ja čeka da Neo pređe na Genode i seL4 (odluka 0001), da bi se pisao jednom, za konačno jezgro.

## Izvori

- Directory Opus 5, APL 1.1: [THIRD_PARTY_NOTICES](https://raw.githubusercontent.com/BlitterStudio/dopus5/master/THIRD_PARTY_NOTICES.md), [Generation Amiga, april 2026.](https://www.generationamiga.com/2026/04/22/the-return-of-directory-opus-amigas-legendary-file-manager-gets-new-life/)
- Galileo: [github.com/Hagbard-Celin/galileo](https://github.com/Hagbard-Celin/galileo)
- Scalos: [Wikipedia](https://en.wikipedia.org/wiki/Scalos), [SourceForge](https://sourceforge.net/projects/scalos/)
- Janus-UAE: [SourceForge](https://sourceforge.net/projects/janus-uae/)
- FS-UAE: [fs-uae.net](https://next.fs-uae.net/download/source)
