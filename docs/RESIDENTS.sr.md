# Stanari, i Zmija kao drugi stanar

[English](RESIDENTS.md) · **Srpski** · Plan, 27. novembar 2026.

Dijana je prvi stanar. Specifikacija (poglavlje 8c) već imenuje prvog kandidata za drugog: zmiju iz pesme „Ide zmija” DJ Nick DeeTronic-a, kao plesnog stanara uz Gramofon. Vlasnik je razgovarao sa DJ Nickom i on pristaje na **test build**.

Specifikacija navodi i temu Kristal (DJ Nick, Nikola Tomić, Deetronic Design Studio, za AmigaOS 4) kao treći globalni stil, čiji je domaćin Zmija. Pisana licenca za Zmiju treba da kaže da li obuhvata i temu Kristal, njen miš, ikone, pozadine i zvuk paljenja.

## Šta je stanar

Paket animacija, zvukova i pravila napravljen alatom iz Neo SDK. Samo prima događaje od sistema; ne vidi fajlove ni mrežu i ne prikazuje reklame. Stanari se kupuju jednom, bez pretplate; stanari zajednice su besplatni pod slobodnom licencom; poznati likovi traže zvaničnu licencu.

## Šta treba za Zmiju

| # | Potreba | Napomene |
| --- | --- | --- |
| 1 | **Pisana licenca** | Nacrt je u privatnom folderu (`nbpos-private/ZMIJA-LICENCA-NACRT.sr.md`): obuhvata samo lik, samo za ovaj OS, prvo test faza. Pre potpisivanja treba advokat za intelektualnu svojinu u Srbiji |
| 2 | **Ko drži prava** | Lik, pesma (kompozicija, tekst) i snimak mogu da pripadaju različitim ljudima: koautorima, izdavaču, producentu. Uzeti kratku pisanu potvrdu ko je vlasnik čega |
| 3 | **Crtež** | Ko crta Zmiju, i pisan prenos ili licenca od tog autora. Pixel, vektorska i animirana verzija, kao kod Dijane |
| 4 | **Pesma** | Najbezbednije: ne isporučivati je. Zmija pleše uz sve što svira u Gramofonu (prepoznavanje ritma). Ako se želi kratak isečak, za kompoziciju i za snimak treba posebna licenca |
| 5 | **Okvir za stanare** | Tok događaja, format paketa (sprajt-list, atlas, fajl pravila), alat u Neo SDK, izbor u Podešavanjima, stavka u prodavnici. Još ne postoji; dolazi uz radnu površinu |
| 6 | **Pravila test builda** | Zatvoreno testiranje sa imenovanim testerima, označeno TEST, sa datumom isteka. Fajlovi Zmije žive u **odvojenom paketu sa sopstvenim fajlom licence** i **ne** idu u javni GPL repozitorijum |
| 7 | **Licence držati odvojeno** | GPL pokriva Neo-ov kod. Ne daje pravo na Zmiju, a licenca Zmije ne daje pravo nad kodom |

## Redosled

Prvo okvir za stanare sa Dijanom; zatim Zmija kao drugi paket, u privatnom test buildu; komercijalni uslovi se određuju tek posle testa.
