# Zajedno: video, pozivi, strimovanje i stvaranje sa drugima

[English](TOGETHER.md) · **Srpski** · Plan, 28. novembar 2026. · **[lite]**

Vlasnik je pitao šta NBPOS nudi današnjoj deci i odraslima za video, strimovanje i veb sastanke, i šta najviše doprinosi produktivnosti, dobroj zabavi i zdravoj vezi među ljudima uz pomoć računara, uz jaču edukativnu i kreativnu stranu. Ova strana je odgovor i plan.

## Šta najviše pomaže (ovim redom)

1. **Stvarati nešto zajedno** je bolje nego gledati nešto sam. Zajednička mapa uma za školski projekat, priča koju dvoje pišu u isto vreme, LAN igra u jednoj sobi: računar je sto oko kog ljudi sede, a ne zid između njih.
2. **Razgovor bez posrednika.** Pozivi bez naloga, bez reklama i bez snimanja od strane neke firme; dovoljan je link.
3. **Stvarati, a ne samo trošiti.** Snimanje, montaža, miksovanje i strimovanje sopstvenog rada uči više od bilo kakvog fida.
4. **Fid koji sami birate.** Pretplate koje ste izabrali, bez beskrajnih preporuka, bez automatskog puštanja sledećeg videa (pravilo koje već važi za vesti: bez guranja i bez propagande).
5. **Odmor.** Dijanina pauza 20-20-20 i vreme za „Laku noć” važe i ovde ([KIDS.sr.md](KIDS.sr.md)).

## Novi programi [lite]

Svi su slobodan softver koji danas radi na x86_64 i ARM-u, prvo kao Flatpak programi u Linux kutiji, svaki sa jednostavnim imenom i redom „zasnovano na” ([APP-NAMES.sr.md](APP-NAMES.sr.md)).

| Neo ime | Radi | Zasnovano na | Licenca (za proveru) |
| --- | --- | --- | --- |
| **Kanal** | YouTube i PeerTube video bez reklama i bez naloga: lokalne pretplate, liste, bez praćenja, SponsorBlock, preuzimanje za kasnije | Sada **FreeTube** (desktop pandan NewPipe-a); kasnije Neo Kit interfejs preko **yt-dlp** i **mpv**, kao što radi **Pipeline** na GNOME-u | FreeTube AGPL-3.0, yt-dlp Unlicense, mpv GPL-2.0+/LGPL, Pipeline GPL-3.0 |
| **Sastanak** | Video pozivi i učionice bez naloga; dovoljan je link ili ime sobe; može da radi na kućnom ili školskom NBPOS računaru za lokalnu mrežu | **Jitsi Meet** | Apache-2.0 |
| **Emisija** | Snimanje ekrana i strimovanje (časovi, igre, muzika), scene i natpisi | **OBS Studio** | GPL-2.0+ |
| **DJ pult** | DJ miksovanje sa dva ili četiri deka, kontroleri, snimanje miksa | **Mixxx** | GPL-2.0+ |
| **Pero zajedno** | Pisanje jednog dokumenta u isto vreme na lokalnoj mreži, „Google Docs bez Google-a” | Pero plus CRDT sloj (Yjs ili Automerge, MIT); za pune kancelarijske dokumente Collabora Online na jednom NBPOS računaru u kući | MIT; Collabora Online MPL-2.0 |
| **DreamCatcher zajedno** | Jedna mapa uma koju menja više ljudi na lokalnoj mreži | [DREAMCATCHER.sr.md](DREAMCATCHER.sr.md) | MIT sloj |

### Postoji li NewPipe za desktop?

Postoji. Sam **NewPipe** je Android aplikacija (GPL-3.0); radiće u Android sobi (verzija 1.1). Na desktopu je najbliži pandan **FreeTube**: bez reklama, bez Google naloga, pretplate se čuvaju na vašem računaru, po želji preko Invidious-a. **Pipeline** (GTK) radi isto za YouTube i PeerTube zajedno. Kanal počinje kao FreeTube i postaje Neo Kit program na yt-dlp i mpv, pa YouTube i **PeerTube** (decentralizovani video, AGPL-3.0) izgledaju isto u jednoj listi.

**Iskreno o tome.** YouTube-ovi uslovi korišćenja ne dozvoljavaju tuđe plejere koji preskaču reklame, a YouTube često menja sajt, pa takvi plejeri povremeno prestanu da rade dok se ne poprave. Zato je Kanal **zaseban program** koji korisnik instalira iz Riznice, a ne deo osnovnog sistema, i Neo ne obećava da će uvek raditi. Strana „O programu” u Kanalu predlaže i da autore koje gledate podržite direktno (njihove strane, Patreon, donacije na PeerTube-u).

## Za decu (Dečja soba)

- **Kanal:** samo kanali koje je dodala odrasla osoba, bez pretrage, komentara i preporuka; mogu se dodati PeerTube instance napravljene za škole.
- **Sastanak:** samo sobe i kontakti koje je odobrila odrasla osoba; svetlo kamere i mikrofona i Bravin dnevnik pokazuju kad su uključeni.
- **Emisija i DJ pult:** snimanje radi, strimovanje uživo na internet je isključeno dok ga odrasla osoba ne uključi.
- **Programi „zajedno”** podrazumevano rade na lokalnoj mreži; ništa ne izlazi iz kuće ili učionice.

## Mreža i privatnost

Programi „zajedno” se pronalaze na lokalnoj mreži (mDNS), dele fajlove preko `p9fs` ([PLAN9.sr.md](PLAN9.sr.md)) i ne treba im server na internetu. Brava pokazuje koji program priča sa kojim računarom.

## Procena i redosled

1. Flatpak verzije Kanala (FreeTube), Sastanka, Emisije i DJ pulta, čim prorade Linux kutija, zvuk i kamera: oko 2 čovek-meseca za pakovanje, pravila Brave i ograničenja Dečje sobe.
2. Pero zajedno i DreamCatcher zajedno posle Pera i DreamCatcher-a: oko 4 čovek-meseca za zajednički CRDT sloj.
3. Kanal kao Neo Kit program na yt-dlp i mpv: oko 2 čovek-meseca.
