# Neo Boing Platinum OS Lite za decu

[English](KIDS.md) · **Srpski** · Plan, 27. novembar 2026.

Verzija Lite-a sa pojednostavljenim interfejsom i manje funkcija, za decu. **Nije poseban sistem**: to je isti Lite sa profilom „Dečja soba” kao podrazumevanim, manjim skupom programa i zaključanim funkcijama za odrasle. Specifikacija već ima delove (Lako režim, Lako raspored, Dečja soba, veliki profil, „Briši bez straha”, Dijanina sedam zadataka); ova strana ih sklapa. Ime je otvoreno („Neo Deca” je samo radni naziv).

## Kako izgleda

- Počinje u **Lako rasporedu**: velika Polica sa šest stvari i sat, veliki tasteri, reči pored svake ikone.
- Jedna glavna radnja po ekranu, kako traži specifikacija; traka menija sa malo stavki.
- Dijana je prisutnija, a pomoć je podešena na „Nagoveštaj”.
- Brisanje bez pitanja, sa trakom „Vrati” i Kantom (već određeno).
- Nema Pro imena ni tehničkih kodova na ekranu: NEO kod stoji iza tastera „detalji za odrasle”.

## Dva uzrasta (predlog, vlasnik prihvata ili odbacuje)

| Uzrast | Okvirno | Sto |
| --- | --- | --- |
| Mali | 4 do 7 | Šest pločica: crtanje, muzika, priče, igre, slike, Dijana. Bez pogleda na fajlove, ništa ne traži kucanje da bi počelo |
| Veliki | 8 do 12 | Dodaje pisanje, Neo BASIC i Neo Karte, fajlove u jednostavnom pogledu, igru kucanja, i bezbedan pregledač ako ga odrasli uključi |

## Šta je isključeno ili zaključano

Iza ključa za odrasle (Ključevi u Bravi):

- Prekidač Pro, Neo DOS i Školjka (kurs „Komandna linija za decu” u Neo školi i dalje radi unutar kursa).
- Instalacija programa, Linux box, Windows i Android sobe, torenti.
- Pregledač, pošta i ćaskanje, osim ako ih odrasli uključi; pregledač može da se ograniči na spisak sajtova koje odrasli izabere.
- Igra poker (samo profil 18+).
- Promena profila ili podešavanja sistema.

Brava već pokazuje šta napušta računar, pa to odrasli mogu da vide. Bez naloga, bez reklama, bez telemetrije, kao na svakom Neo-u.

## Programi u skupu

Platno i PhotoCraft u jednostavnom režimu, Pero (veliko, malo alata), Neo BASIC, Neo Karte, Kristal, Pasijans i Minolovac, Velemajstor, Šumski Bane, Gramofon, Album i pregledač, Listaj, igra kucanja, čuvar ekrana Dijanin dan, **Školsko zvono** (Neo školska avantura, omaž Spectrum igri *Skool Daze*, sa pitanjima iz Neo Škole: [GAMES.sr.md](GAMES.sr.md)), **DreamCatcher** u jednostavnom režimu sa velikim čvorovima i nalepnicama ([DREAMCATCHER.sr.md](DREAMCATCHER.sr.md)) i, kad ih odrasla osoba uključi, **Kanal** samo sa odobrenim kanalima i **Sastanak** samo sa odobrenim kontaktima ([TOGETHER.sr.md](TOGETHER.sr.md)). U školi važi Režim učionice (verzija 1.1).

## Vreme i odmor

Dijanina pauza 20-20-20 i vreme „Laku noć” koje podešava odrasli. To su podsetnici i izbori zaključavanja za odrasle, ne obećanje kontrole.

## Iskreno o granicama

Nijedan filter nije potpun, a odrasla osoba u blizini i dalje je potrebna. Pre izdanja proveriti pravila o podacima dece u zemljama u kojima se isporučuje. Podaci samo na računaru i nepostojanje naloga pomažu, ali to je pravno pitanje za pregled pre izdanja koji specifikacija već planira.

## Kako se gradi

- Isti ISO sa unapred postavljenim izborom: izbor pri prvom paljenju „Ko koristi računar: Dečja soba” upisuje se kao podrazumevan, a skup programa dolazi sa spiska. U smislu [BUILD-ISO.sr.md](BUILD-ISO.sr.md) to je profil, na primer `scripts/profiles/kids.env`.
- Prvo traži radnu površinu, Lako raspored i Ključeve u Bravi, pa dolazi posle rada na radnoj površini u redu čekanja iz [HANDOFF.sr.md](HANDOFF.sr.md).
