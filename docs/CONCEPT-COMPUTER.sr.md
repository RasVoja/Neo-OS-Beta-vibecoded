# Dve odvojene stvari: sistem i koncept idealnog računara

[English](CONCEPT-COMPUTER.md) · **Srpski** · Plan, 27. novembar 2026.

## Odluka

Vlasnik je odlučio da razdvoji dve stvari koje su u dokumentima, demoima i video snimcima bile pomešane:

1. **Operativni sistem** (Neo Boing Platinum OS, sa Neo Boing Platinum OS Lite kao prvim izdanjem). Radi na postojećem običnom x86_64 i ARM hardveru i **dominantna** je stvar u repozitorijumu, demoima i video snimcima.
2. **Koncept idealnog računara** (Neo 500, 2000, 4000, Neo paket, FPGA jezgra, Sunce stanica i tako dalje). Računar koji bi se isporučivao sa sistemom, prikazan **potpuno odvojeno**, označen kao konceptni hardver. U njemu je **Neo Boing Platinum samo jedna dodatna mogućnost**, a ne identitet celine.

**Ime.** Sistem je **Neo Boing Platinum OS** (odlučeno 18. novembra 2026) i stoji sam za sebe na bilo kom PC-ju ili ARM ploči. Na konceptnom računaru (konceptni hardver) unapred instaliran Neo Boing Platinum OS je jedna dodatna mogućnost mašine, pored njenih čipova, kertridža i ROM-a; mašina nikad nije uslov za sistem.

## Pravilo za svaki dokument, demo i video

| Pitanje | Ako da | Ako ne |
| --- | --- | --- |
| Radi li danas, ili može da radi na običnom PC-ju ili ARM ploči? | Pripada **sistemu**. Glavne strane, glavni demoi | Sledeće pitanje |
| Traži li hardver koji ne postoji (Neo ploče, čipovi, FPGA)? | Pripada **konceptu**. Odvojeno područje, oznaka „Koncept” | Pitati vlasnika |

Svaka konceptualna strana, demo i video pokazuje vidljiv natpis „Koncept: traži hardver koji ne postoji”. Strane sistema nikad ne pominju konceptualni hardver kao uslov. Sistem se pokazuje kako radi u QEMU-u ili na pravom PC-ju; koncept se pokazuje kao crteži.

## Predloženi raspored repozitorijuma

```
README.md             prvo sistem; jedna kratka uputa na koncept
docs/                 sistem: stanje, mogućnosti, planovi, pomoć (ovaj folder)
docs/html/…/demo/     demoi sistema (Ormar, Neo DOS, Pločice, Zgusni, Prvi Sto, jezici …)
concept/              idealni računar: svoj README, pregled, brošura, tehnička knjiga
concept/demo/         hardverski demoi (ploča, paket, Sunce stanica, FPGA)
```

Šest PDF-ova i HTML stranice koji su sada u `docs/pdf` i `docs/html` mešaju oba. Sređuju se u dva koraka:

1. **Rasecati** svaki na deo o sistemu i deo o konceptu (specifikacija i tehnička knjiga imaju jasan softverski i hardverski deo).
2. **Premestiti** hardverske delove i hardverske demoe u `concept/`.

**Urađeno 27. novembra 2026 (prvi korak):** README sada stavlja sistem na prvo mesto (korisnički vodič, demoi sistema, dokumenti o sistemu), a pregled, brošuru, tehničku knjigu, stranu o identitetu i demo „Neo za svakog” navodi u posebnom odeljku pod naslovom „konceptni hardver”. Svaki pomen Neo 500/2000/4000 u stranama planova nosi oznaku „(konceptni hardver)”. Sami dokumenti još nisu rasečeni ni premešteni; to je sledeći korak, fajl po fajl, da se ništa ne izgubi.

## Glavni demoi sistema

Već su u repozitorijumu i o sistemu: Ormar, Ormar Magelan, Neo DOS, Pločice, Zgusni, Prvi Sto, jezici, škola, vesti, Claude pomoć, veza sa telefonom i beta priručnik. Oni postaju prvi red u spisku demoa.

## Video snimci

Repozitorijum nema video fajlove, samo dokumente, veb demoe i snimke ekrana. Vlasnikovi snimci čuvaju se van njega i za njih važi isto pravilo: video snimci sistema pokazuju sistem kako radi; konceptualni video je odvojen, sa natpisom.

## Koncept video snimci (primljeni 27. novembra 2026)

Šest vlasnikovih koncept snimaka (ukupno oko 7 minuta): Neo promo, Neo identitet, Neo u svakodnevici, Neo ideje na delu, Neo aplikacije u radu i „Neo od klika do čipa”. Prikazuju koncept računara (ploča sa ARM-om, FPGA i VSA-6000, Studio u kutiji) zajedno sa mogućnostima sistema (Gramofon, Brava, igre, montaža). To je **koncept** materijal; razvrstava se fajl po fajl sa vlasnikom pre nego što se poveže iz README-a.

### Šta snimci znače za sistem (po prvom pogledu na kadrove)

- **Pripada sistemu (Lite, demo materijal):** Prvi Sto i izgled Neo Srebro, Gramofon, Brava (privatnost), polica sa igrama, montaža, tri zvučne teme (Kristal, Drvo, Čip), tri pisma sa ćirilicom i latinicom, pločice sa živim podacima, stanar Dijana (raspoloženja: dobro jutro, greška, spavanje).
- **Samo koncept računara (označiti kao koncept):** ploča sa ARM-om, FPGA i VSA-6000, ROM i NVMe kertridži, solarni panel i baterija, modeli Neo 500/2000/4000, „Studio u kutiji”.
- Snimci koji mešaju oba treba da se iseku ili da dobiju oznaku „koncept” pre nego što postanu glavni demo sistema. Gledani su kadrovi samo dva od šest snimaka; ostalo nije provereno.
