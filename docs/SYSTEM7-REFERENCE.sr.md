# System 7 uzori: izgled prozora, fontovi i emulator Classic Mac-a u Arkadi

[English](SYSTEM7-REFERENCE.md) · **Srpski** · Plan, 27. novembar 2026.

Vlasnik je tražio da se tri stvari preuzmu iz projekta Kelsidavis/System7. Ovde je šta se pokazalo da taj projekat jeste i šta Neo radi sa svakom tačkom.

## Šta je Kelsidavis/System7

Reimplementacija Apple Macintosh System 7 otvorenog koda za moderan x86 hardver, pisana u C-u, podiže se preko GRUB2 i Multiboot2, pod MIT licencom (prema javnoj statistici repozitorijuma). Sam sebe opisuje kao eksperimentalni dokaz koncepta, napravljen uz pomoć jezičkog modela. **Nije CSS projekat.**

Dve opreznosti: prikazuje font Chicago i dugin Apple logo, koji pripadaju Apple-u. Neo ne kopira ni jedno ni drugo. Neo uzima ponašanje i duh, kako piše specifikacija, nikad Apple-ovu grafiku, fontove ni zaštitne znake.

## 1. CSS za klasične prozore (naslovne trake, radio dugmad, polja za izbor, klizači)

CSS biblioteke za ovaj izgled su zasebni projekti (na primer system7.css). Njihove licence i licence fontova koje isporučuju nisu potvrđene, pa se **ne kopira ni kod ni font**. Umesto toga `docs/assets/first-desk.css` je napisan ispočetka za temu Prvi Sto: prugasta naslovna traka, polja za zatvaranje i zum, jednobitna radio dugmad i polja za izbor, zaobljeni tasteri sa dvostrukim okvirom za podrazumevani, i jednobitni klizači. `first-desk-demo.html` ih prikazuje. Služi za dokumente, demoe i veb stranicu; pravi sistem crta temu u Neo Kitu.

*Još nije pogledano u pregledaču; proveriti pre upotrebe.*

## 2. Stil fonta i ikona za dokumentaciju i demoe

Neo ima sopstveni bitmap font „Neo Sistem”, planiran u latinici i ćirilici, i sopstvene pikselske ikone. Chicago i Geneva pripadaju Apple-u; Neo ih ne koristi niti njihove kopije. Dok Neo Sistem ne postoji, dokumenti i demoi koriste sistemski font sa fiksnom širinom. Osećaj (debeli potezi od jednog piksela, gusto razmaknuto) je vodilja za novi font, ne oblici slova.

## 3. Emulator Classic Mac-a u Arkadi

Arkada bira način pokretanja programa, a korisnik bira program. Classic Mac OS tu postaje još jedan sistem, na način na koji se danas pokreće Amiga.

| Emulator | Licenca | Pokreće |
| --- | --- | --- |
| Basilisk II | GPLv2 | 68k Mac-ove, System 7 do 8.1 |
| SheepShaver | GPLv2 | PowerPC Mac OS 7.5.2 do 9 |
| Mini vMac (i kao libretro jezgro) | GPLv2 | kompaktne Mac-ove, System 3 do 7.5 |
| Snow | MIT | 68k Mac-ove (128K do Mac II) |
| Advanced Mac Substitute | proveriti | stare Mac programe bez Apple ROM-a i sistemskog softvera |

**Ne isporučuje se:** Apple ROM slike i Apple sistemski softver. Većini ovih emulatora trebaju oba. Arkada traži od korisnika njegove kopije i pokazuje običan opis, a zatim pokreće program kao i za svaki drugi sistem. Emulatori idu kao posebni programi, po pravilu o licencama iz [AMIGA.sr.md](AMIGA.sr.md).

## Redosled

CSS i demo su gotovi odmah. Emulatori dolaze uz Arkadu, posle Linux boxa; Advanced Mac Substitute vredi prvo probati jer ne traži Apple fajlove.
