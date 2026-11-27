# Kreativni programi: FilmCraft, PhotoCraft i Album

[English](CREATIVE-APPS.md) · **Srpski** · Plan, osveženo 27. novembra 2026.

Sva tri su **[lite]**: rade na običnom x86_64 i ARM hardveru i ne čekaju Neo hardver. Svaki prvo radi kao zreo slobodan program u **Linux kutiji** (preko Flatpaka), pod jednostavnim Neo imenom, sa redom „zasnovano na” na uvodnom ekranu, u O programu i u Pomoći ([APP-NAMES.sr.md](APP-NAMES.sr.md)). Neo Kit interfejs dolazi kad radna površina bude postojala. Licence se moraju ponovo proveriti u repozitorijumu svakog projekta pre ubacivanja bilo kog koda.

| Prikazuje se kao | Zasnovano na | Licenca (proveriti ponovo) | Šta radi |
| --- | --- | --- | --- |
| **FilmCraft** | Kdenlive | GPL-3.0+ (neki delovi GPL-2.0+) | Video editor sa vremenskom linijom |
| **PhotoCraft** | GIMP (raster) i Inkscape (vektor) | GPL-3.0+ / GPL-2.0+ | Editor slika, jedna stavka za fotografije i crteže |
| **Album** | gThumb | GPL-2.0+ | Pregledač i organizator slika |

## FilmCraft: video editor (Kdenlive)

- **Plan.** FilmCraft je Kdenlive pod Neo imenom. Zreo je, radi na x86_64 i ARM-u i ima ga na Flathubu, pa može da stigne sa prvim Lite-om koji ima Linux kutiju. Ovo zamenjuje raniju stavku „Montaža (Kdenlive)” iz specifikacije; program je isti, ime koje se vidi je FilmCraft.
- **Neo dodaci.** Otvara fajlove iz Ormara i Albuma, čuva gotove snimke u korisnikov folder Filmovi, koristi Tipove podataka za formate kad budu postojali i prikazuje Neo strane pomoći.
- **Kasnije.** Neo Kit interfejs („NeoGUI”) preko istog motora (MLT). Rust video editor projekta ArtCraft, koji se takođe zove FilmCraft, prati se kao mogući kasniji motor; još je u ranoj fazi (verzije 0.2 do 0.5 u izveštajima).
- **Prvo treba.** Linux kutija, zvučni server, GPU ili softversko iscrtavanje preko Mese.

## PhotoCraft: editor slika (GIMP i Inkscape)

- **Plan.** PhotoCraft je tanak Neo omotač sa jednom stavkom na Polici i jednim prvim ekranom: „Uredi fotografiju” otvara GIMP, „Crtaj ili dizajniraj” otvara Inkscape. Korisnik vidi jedan program, PhotoCraft; O programu i Pomoć navode oba programa na kojima je zasnovan.
- **Neo dodaci.** Zajednička lista skorašnjih fajlova, ista mesta za Otvori i Sačuvaj, Neo strane pomoći sa slikama i Lako profil koji krije stručne menije gde programi to dozvoljavaju.
- **Kasnije.** Neo Kit interfejs; mesto „Kist” u paketu Radionica (Krita) ostaje kao opcija za crtanje na tabletu.
- **Prvo treba.** Linux kutija; oba programa su na Flathubu.

## Album: pregledač slika (gThumb)

**Odlučeno 27. novembra 2026: pregledač slika je gThumb, prikazan kao Album.**

- gThumb je slobodan softver (GPL-2.0+), pa može da se isporuči u Neo-u i da se menja.
- Dobija osećaj biblioteke, albuma, foldera i brzih ispravki u Neo stilu: vremenska linija fotografija, albumi kao Blokovi, okretanje, sečenje i ispravka svetla jednim klikom. Veće izmene otvaraju se u PhotoCraftu.
- Album je i podrazumevani pregledač slika otvorenih iz Ormara, pošte i veba.
- Prvo treba: Linux kutija (ili kasnije izvorni GTK prenos) i Tipovi podataka za formate.

## Redosled

1. Linux kutija sa Flatpakom.
2. Album (gThumb), najmanji i najviše korišćen.
3. FilmCraft (Kdenlive) i PhotoCraft (GIMP i Inkscape).
4. Neo Kit interfejsi za sva tri, posle radne površine.
