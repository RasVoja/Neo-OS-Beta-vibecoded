# Klasični Mac izgled i programi na Neo-u

[English](CLASSIC-MAC.md) · **Srpski** · Plan, 18. novembar 2026.

Neo-ov izgled uzima što više ideja iz klasičnog Mac OS-a (vidi specifikaciju, poglavlje 8b, i retro temu „Prvi Sto”). Ova strana planira dve mogućnosti oko otvorenog projekta [Kelsidavis/System7](https://github.com/Kelsidavis/System7). To je ponovo napisan Macintosh System 7 za x86 koji se podiže preko GRUB-a i Multiboot2, kao i Neo Lite sada.

## Šta je taj projekat (provereno 18. 11. 2026.)

- **Licenca:** MIT, prema opisu projekta. Pre nego što bilo koji njegov kod uđe u Neo, treba pročitati njegov LICENSE fajl i `docs/DERIVATION_APPENDIX.md`.
- **Kako je nastao:** obrnutim inženjeringom System 7 binarnih fajlova, uz Ghidru i jezički model. U javnim komentarima se spori da li delovi potiču od Apple-ovog koda, pa njegovo **poreklo nije razjašnjeno**.
- **Stanje:** autor ga zove „kostur koji se podiže”. Radi u QEMU-u i još nije pun sistem.

Iz toga slede dva pravila. **Ideje i ponašanje** iz njega uzimamo slobodno. **Kod** uzimamo tek posle provere porekla (stavka C3). Neo takođe nikad ne isporučuje Apple-ova imena, fontove, ikone ni crteže; izgled je Neo-ov sopstveni crtež u klasičnom duhu.

## Stavke

| # | Stavka | Šta Neo dobija | Procena |
| --- | --- | --- | --- |
| C1 | **Klasični prozori i kontrole za „Prvi Sto”** | Okviri prozora, naslovne trake sa prugama, kućice za zatvaranje i uvećanje, trake za pomeranje, dugmad, polja za potvrdu i meniji, koje neo-gfx crta u klasičnom stilu kao Neo-ov sopstveni crtež. Isti izgled i kao CSS za HTML demoe | 6 č-m |
| C2 | **Klasični Mac programi u Arkadi** | Arkada pokreće klasične Mac programe i igre preko zrelih emulatora: Basilisk II i SheepShaver (GPL-2.0) za 68k i PowerPC, i Mini vMac (GPL-2.0) za prve Mac-ove. Svaki radi kao zaseban program i traži Mac ROM koji korisnik poseduje | 6 č-m |
| C3 | **Ponovo napisan System 7: procena** | Čitanje licence i beleški o poreklu. Ako je poreklo čisto, dodaje se u Arkadu kao način da se klasični programi pokreću bez Apple ROM-a, a ponašanje njegovog upravljača prozorima služi kao uzor za C1 | 1 č-m |

**Ukupno:** oko 13 č-m, oko 2 100 sati. Deo C1 se već računa u radu na temama iz specifikacije.

## Redosled

1. **Uz M3** (teme, srpski i engleski kompletni): C1.
2. **Posle M3**, uz Amiga emulaciju u Arkadi (vidi [AMIGA.sr.md](AMIGA.sr.md)): C2 i C3.

## Izvori

- [Hackaday, 29. septembar 2025: Macintosh System 7 prenet na x86 uz pomoć jezičkog modela](https://hackaday.com/2025/09/29/macintosh-system-7-ported-to-x86-with-llm-help)
- [OSS Insight: Kelsidavis/System7](https://ossinsight.io/analyze/Kelsidavis/System7)
