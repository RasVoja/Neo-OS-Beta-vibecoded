# Muzika i svakodnevni programi: HippoPlayer, qBittorrent, Hydrogen, trekeri

[English](MUSIC.md) · **Srpski** · Plan, 27. novembar 2026.

Odluke vlasnika projekta, uz ono što svaka traži da bi mogla da počne. Licence su kako su nađene 27. novembra 2026. i pre ubacivanja bilo kog koda moraju ponovo da se pročitaju u fajlovima samog projekta. NBPOS je GPLv3 ili novija; program pod drugom licencom ide kao **poseban program** (pravilo je u [AMIGA.sr.md](AMIGA.sr.md)).

**Obim: svih šest stavki je [lite].** Nijedna ne traži Neo hardver; rade na običnim x86_64 i ARM računarima, prvo preko Flatpaka u Linux boxu, kasnije izvorno gde se isplati. Zvučni čip konceptnog računara (konceptni hardver) samo bi ih ubrzao. qBittorrent je i način na koji Gramofon i treker preuzimaju velike slobodne muzičke zbirke, kao što su albumi pod Creative Commons licencom i arhive modula, a Brava ga ograničava na folder za preuzimanje.

| # | Stavka | Licenca (proveriti ponovo) | Plan | Prvo treba |
| --- | --- | --- | --- | --- |
| M1 | **HippoPlayer** kao audio engine za Gramofon | MIT i Apache prema Arch paketu; dodaci poput UADE imaju svoje licence | Uzima se jezgro plejera i njegovi dodaci za Amiga, C64 i čip formate. Neo-ov Gramofon zamenjuje njegov Qt prozor. Uklapa se u „Paula režim” za MOD muziku iz specifikacije | Zvučni drajver i softverski mikser; Linux box dok jezgro ne bude građeno nativno |
| M2 | **qBittorrent** kao torrent aplikacija | GPL, verziju proveriti | Qt program, pa radi u Linux boxu. Brava mu daje mrežu i jedan folder za preuzimanje (Sanduče), ništa drugo | Mrežni stek, Linux box, Brava |
| M3 | **Hydrogen** bubnjarska mašina | GPL-2.0 ili novija | Poseban program u Linux boxu, zvuk preko Neo zvučnog servera | Zvučni server, Linux box |
| M4 | **Beatmaker sa jezičkim modelom** | biće izabrano | Lokalni model (bez oblaka) piše bubnjarske obrasce iz tekstualnog opisa i predaje ih Hydrogenu kao fajl pesme ili MIDI. Uvek prvo pregled, uvek može da se poništi. Na Lite-u radi na procesoru sa malim modelom | M3, okruženje za lokalne modele (treba i za Pitaj Neo) |
| M5 | **Treker** (OpenMPT sa mogućnostima Schism Trackera i MilkyTrackera) | OpenMPT BSD-3-Clause; Schism i MilkyTracker GPL (proveriti) | **Odlučeno 27. novembra 2026: Neo treker**, građen na libopenmpt, koji dodaje Schism-ov i MilkyTracker-ov rad sa tastature i mogućnosti. OpenMPT radi samo na Windowsu; pokretanje u Windows boxu ostaje opciona rezerva | Zvučni server, MIDI portovi |
| M6 | **Uzajamna razmena** trekera, Hydrogena i beatmakera | n/a | Živi MIDI između programa preko imenovanih Exec-N portova (`midi.*`, plus USB MIDI uređaji). Stemovi kao WAV fajlovi: svaki program izvozi u jedan zajednički folder Stemovi, ostali odmah vide novi fajl (okidač u Ormaru) i mogu da ga uvezu | M3 i M5, portovi za poruke (gotovo) |

## Redosled

1. Zvučni drajver i zvučni server (sve ispod traži zvuk).
2. M1 HippoPlayer u Gramofonu, prvi čujan rezultat.
3. Mreža, Linux box, zatim M2 qBittorrent.
4. M3 Hydrogen, M5 treker, M6 razmena, pa M4 beatmaker.

## Otvorena pitanja

- Koji Neo zvučni API vide programi u Linux boxu (kompatibilan sa PipeWire-om, kako piše u specifikaciji)?
- Graditi jezgro HippoPlayer-a nativno na Neo-u ili ga trajno držati u Linux boxu?
- Koji model i koliko velik za M4 na računaru bez AI čipa?
