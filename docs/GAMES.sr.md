# Igre u Lite-u

[English](GAMES.md) · **Srpski** · Plan, 27. novembar 2026.

Vlasnik je tražio da se igre, i što više ranijih Pro mogućnosti, prebace u **Lite** verziju. Pravilo: igra ulazi u Lite ako radi u softveru na običnom x86_64 ili ARM hardveru. Igra koja traži Neo čip ili FPGA jezgro isporučuje se kao **softverska verzija**; hardverska izloga ostaje concept art na konceptnom računaru (konceptni hardver, [CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md)).

Preuzeto sa strane Neo igre iz pregleda. „Originalno” znači originalna igra žanra; licencirane igre traže pisanu licencu nosioca prava pre isporuke.

## Naše Neo igre [lite]

| Igra | Žanr | Izvor | Plan za Lite |
| --- | --- | --- | --- |
| Velemajstor | Šah | Originalno, motor Stockfish (GPL) | **Lite.** Motor i tabla u Neo Kitu; 3D tabla samo ako radna površina ima 3D, inače ravna tabla |
| Pasijans i Minolovac | Paket klasika (Pasijans, Ćelije, Srca, Minolovac) | Originalno, žanr Windows klasika | **Lite, od prvog paljenja.** Najmanja i najjeftinija; prva igra koja se isporučuje |
| Kristal | Padajući blokovi, šestougaoni | Originalna pravila (Tetris je licenciran) | **Lite.** Zvuk i čestice vezani za muziku iz Gramofona |
| Fliper Kosmos | Fliper | Originalno | **Lite.** Softverska fizika, više stolova |
| Šumski Bane | 2D platformer | Originalno | **Lite.** Šarena 2D grafika |
| Blok 70 | Pucačina iz prvog lica | Originalno, Quake motor (GPL) | **Lite, kasnije.** Traži 3D u Linux boxu ili nativni renderer |
| Uskoci | Pomorska avantura, Jadran, 16. vek | Originalno | **Lite, kasnije.** Najveća igra; početi od trgovine i plovidbe |
| Ponoćni poker | Karte, 18+ | Originalno | **Lite, samo odrasli profil.** Skriven na dečjim profilima ([KIDS.sr.md](KIDS.sr.md)); traži profil sa proverenim uzrastom |
| Warblade Neo | Pucačina | **Licencirano**, nosilac prava imenovan u specifikaciji | **Ne dok licenca ne postoji.** Original je radio u 68030 jezgru; softverski remake je moguć kad se licenca dobije |

## Šta je izloga značila i šta se menja

Svaka igra u specifikaciji „pokazuje deo hardvera” (Neuron, VSA-6000, Njutn fizika, jezgro Kameleon). Na Lite-u tog hardvera nema, pa ista igra radi u softveru, a red „Pokazuje: …” se izbacuje iz Lite dokumenata. Na konceptnom računaru (konceptni hardver) ostaje.

## Gotove igre za Lite [lite]

Dodato 27. novembra 2026. Ove igre već postoje kao slobodan softver za x86_64 i ARM Linux, pa ne čekaju ni Neo igre ni Neo hardver. Rade u **Linux kutiji** preko **Flatpaka** (Flathub), pojavljuju se na polici sa igrama pod jednostavnim imenom i sa redom „zasnovano na” ([APP-NAMES.sr.md](APP-NAMES.sr.md)) i dobijaju samo ono što Brava dozvoli: svoj folder za snimljene pozicije, zvuk, a mrežu samo kad igri treba. 3D igre koriste Mesu (na starim PC-jima njen softverski renderer, sporije, ali radi).

| Prikazuje se kao | Zasnovano na | Žanr | Licenca (proveriti ponovo) | Napomene |
| --- | --- | --- | --- | --- |
| Karting | SuperTuxKart | Trke kartinga | GPL-3.0 | 3D; podeljen ekran i igra preko mreže |
| Carstva | 0 A.D. | Istorijska strategija u realnom vremenu | GPL-2.0+ za kod, CC BY-SA za crteže | 3D; traži više memorije, profili za odrasle i tinejdžere |
| Kocke | Luanti (ranije Minetest) | Gradnja sveta od kocki | LGPL-2.1+ za kod, CC BY-SA za medije | Verzija za Dečju sobu sa zaključanom listom servera ([KIDS.sr.md](KIDS.sr.md)) |
| Mehurići | Frozen Bubble | Mozgalica | GPL-2.0 | 2D, mala, dobra za prvo paljenje |
| Wesnoth | Battle for Wesnoth | Fantastična strategija na poteze | GPL-2.0+ | 2D; kampanje uče polako, pristaje nivou Mirno |

## Arkada i emulatori [lite]

Arkada (emulatori i kutije, umesto FPGA jezgara na Lite-u) ih prvo pokreće preko Flatpaka, a kasnije izvorno. Korisnik donosi sopstvene ROM-ove, diskove i fajlove igara; Neo ne isporučuje nijedan.

| U Arkadi | Zasnovano na | Licenca (proveriti ponovo) | Za |
| --- | --- | --- | --- |
| DOS kutija | DOSBox (kandidat je DOSBox Staging) | GPL-2.0+ | Stare PC igre i programi iz DOS-a |
| Kutija za avanture | ScummVM | GPL-3.0+ | Avanture na klik; mnoge besplatne igre (Beneath a Steel Sky, Flight of the Amazon Queen) mogu se ponuditi iz Riznice |
| Retro kutija | RetroArch (libretro jezgra) | GPL-3.0; svako jezgro ima svoju licencu | Konzole i kućni računari; isporučuju se samo jezgra sa slobodnom licencom |
| Classic Mac | iz [SYSTEM7-REFERENCE.sr.md](SYSTEM7-REFERENCE.sr.md) | vidi tamo | Emulatori klasičnog Mac-a |
| Amiga | FS-UAE, iz [AMIGA.sr.md](AMIGA.sr.md) | GPL-2.0+ | Emulator Amige |

Programi pod GPL-2.0-only ili drugim licencama idu kao posebni programi, nikad spojeni sa sistemom (pravilo u [AMIGA.sr.md](AMIGA.sr.md)).

Preuzimanje fajlova igara i muzike preko **qBittorrenta** je takođe Lite program; vidi [MUSIC.sr.md](MUSIC.sr.md) (M2).

## Redosled

1. Gotove igre i Arkada preko Flatpaka, čim Linux kutija proradi (traže Linux kutiju, zvuk i policu sa igrama).
2. Pasijans i Minolovac (traže Neo Kit i radnu površinu).
3. Velemajstor, Kristal, Fliper Kosmos, Šumski Bane.
4. Blok 70, Uskoci, Ponoćni poker.
5. Warblade Neo samo uz licencu.

Igre koriste ista pravila pomoći kao ostali programi ([HELP.sr.md](HELP.sr.md)): jednostavno ime, red „zasnovano na” i prva strana „Kako se igra”.
