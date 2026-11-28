# Piši, pošalji, pričaj: Pero, Pismo i Razgovor

[English](WRITE-SEND-TALK.md) · **Srpski** · Plan, 28. novembar 2026.

Tri programa koji rade kao jedan: **Pero** piše dokument, **Pismo** ga šalje mejlom, **Razgovor** ga šalje drugom Neo korisniku na mreži kao živi fajl. Uz to i školska strana iste ideje u **Školskoj klupi**. Sve ovde je **[lite]**: samo softver, na običnim x86_64 i ARM računarima, Neo hardver nije potreban.

## Zašto HTML i dalje ima smisla

Ručno uređivanje HTML-a zvuči prevaziđeno sada kad AI pravi sajtove, a WordPress radi preko šablona. Ali HTML je i dalje najbolji jezik za ono što nije sajt: **dokumente, elektronske knjige (EPUB je HTML u ZIP-u), mejl (svaki mejl klijent prikazuje HTML), stranice pomoći i štampu**. Otvoren je, čita ga svaki uređaj i radiće i za 30 godina. Zato NBPOS ne pravi editor sajtova, nego uzima dobre navike klasičnih vizuelnih editora i stavlja ih tamo gde pomažu svima.

| Inspirisano sa (platforma, godina) | Šta je radilo dobro | Šta NBPOS uzima |
| --- | --- | --- |
| **Dreamweaver** (Macromedia, kasnije Adobe; Windows i Mac, 1997) | Vizuelni prikaz i kod iste strane jedno pored drugog | HTML režim Pera može da se podeli na stranu i njen izvor, oba se mogu menjati |
| **RocketCake** (Ambiera; Windows i macOS, 2010-te, besplatno izdanje) | Strane koje se prilagođavaju ekranu bez pisanja koda | Gotovi rasporedi (Blokovi) koji lepo izgledaju i na telefonu i na velikom ekranu |
| **MetalWeb** (Amiga, kraj devedesetih) | WYSIWYG HTML editor dovoljno mali za klasičnu Amigu | Mali, brz editor; HTML režim se otvara za manje od sekunde |
| **YAM**, Yet Another Mailer (Amiga, 1995) | Filteri, ARexx port, više identiteta, sve u običnim fajlovima | Pismo: pravila, Lux port, više identiteta, pošta kao obični fajlovi |
| **Iris** (MorphOS, 2020-te) | Moderan, brz mejl klijent na malom sistemu: IMAP, niti, HTML pošta | Pismo: niti, IMAP pre svega, HTML se prikazuje i piše istom mašinom |
| **WookieChat**, IRC klijent (Amiga i MorphOS, 2000-te) | Teme, retro prozor, slike prikazane u razgovoru | Razgovor: teme, retro prozor, slike u tekstu |
| **BeOS Mail** (BeOS, 1998) | Folderi pošte kao sačuvani upiti nad atributima poruka | Pismo: folderi kao sačuvane pretrage |
| **Apple Bonjour / Rendezvous** (Mac OS X, 2002) | Računari se sami nalaze na mreži, bez podešavanja | Razgovor: mDNS otkrivanje, bez naloga |
| **LocalSend** (Windows, macOS, Linux, Android, iOS, 2022) | Fajlovi između obližnjih uređaja bez oblaka | Razgovor: isti protokol, pa se i telefoni mogu priključiti |
| **Veyon**, ranije iTALC (Linux i Windows, 2004 / 2017) | Otvoreno upravljanje učionicom: pregled ekrana i emitovanje | Školska klupa: pregled razreda i emitovanje |
| **Google Classroom** (veb, 2014) i **Notion** (veb, 2016) | Zadaci sa rokom; strane koje mešaju tekst, spiskove i zadatke | Zadaci Školske klupe i DreamCatcher kartice, na lokalnoj mreži, bez naloga |
| **Markdown** (Džon Gruber, 2004), **Typora** (2015) | Običan tekst koji se čita kao dokument; oznake skrivene dok kucaš | Podrazumevani Markdown WYSIWYG režim Pera |

## Pero: tri režima, jedan dokument [lite]

Pero (EN: Quill) dobija tri režima, menjaju se jednim dugmetom na traci alata ili sa Neo + 1/2/3:

1. **Običan tekst** za brze beleške i za kod: font jednake širine, ništa skriveno.
2. **Markdown WYSIWYG** (podrazumevano): vidiš naslove, podebljan tekst, spiskove i slike, a fajl je ispod i dalje običan Markdown. Otkucaš `# ` i pojavi se naslov; oznake se vide samo u redu gde je kursor.
3. **HTML WYSIWYG**: pun raspored, tabele, kolone, stilovi, slike oko kojih teče tekst; podeljen prikaz pokazuje izvor pored strane (navika iz Dreamweaver-a).

Promena režima čuva sadržaj. Pri prelazu iz HTML-a u Markdown Pero upozori pre nego što pojednostavi ono što Markdown ne može da drži (kolone, boje), a Poništi to vraća.

**Izvoz iz istog dokumenta:** HTML (jedan fajl sa slikama unutra ili folder), **PDF** (preko mašine za štampu sistemskog veb prikaza), **EPUB** (za Listaj i svaki čitač knjiga), plus ODT i DOCX preko LibreOffice jezgra koje Pero već koristi.

**Pošalji kao mejl** (Fajl › Pošalji kao mejl, ili prevuci dokument na Pismo na Traci): Pero predaje dokument Pismu kao HTML mejl sa slikama u tekstu (`multipart/related`, svaka slika preko `cid:`), uz deo u običnom tekstu za stare čitače. Pismo se otvori sa tekstom na mestu; dodaš samo adresu. Pametno spuštanje pokaže „Pismo: pošalji kao mejl” pre nego što pustiš.

**Mašina:** Markdown režimi koriste `pulldown-cmark` (MIT) i CommonMark sa GitHub tabelama; HTML režim koristi isti WPE WebKit sistemski veb prikaz kao Vidik, pa ono što napišeš izgleda isto u Peru, Pismu, Vidiku i stranicama pomoći.

## Pismo: pošta sa ljudskim licem [lite]

**Pismo** (EN: Letter) je sistemski mejl program. Zamenjuje raniji radni naziv *Golub* iz specifikacije.

- **WYSIWYG HTML pisanje** istom WebKit mašinom kao HTML režim Pera; podešavanje „samo običan tekst” za ljude i liste koje to žele.
- **Šabloni:** pismo, račun, obaveštenje škole, bilten, kao Blokovi (šabloni u Lisa stilu); tvoj šablon je bilo koje pismo sačuvano kao Blok.
- **Prilozi koje vidiš:** slike, PDF-ovi i dokumenti imaju pregled u samom pismu (preko Datatypes) pre slanja i pri prijemu; ne moraš da čuvaš fajl da bi ga video.
- **Lokalni nacrti:** svako pismo se čuva dok kucaš, na tvom disku, i kad je mreža isključena; ništa ne ide na server pre nego što pritisneš Pošalji. Kašnjenje slanja od 30 sekundi omogućava da vratiš pismo.
- **Pravila i Lux port** po uzoru na YAM: razvrstavanje, označavanje, odgovor i prosleđivanje po pravilima; skripte iz Neo DOS-a i drugih programa.
- **Folderi kao sačuvane pretrage** (ideja iz BeOS Mail-a), niti i zajedničko sanduče po uzoru na Iris.
- **Pošalji Neo korisniku:** umesto adrese izabereš osobu iz Razgovora. Dokument ide preko Razgovora kao **živi fajl**: primalac ga otvori u Peru i menja ga, a oboje vide izmene (isti CRDT sloj kao Pero zajedno). Uz to može da ide i obična kopija mejlom za ljude koji nisu na NBPOS-u.
- **Protokoli:** IMAP i SMTP prvo, JMAP kasnije; OpenPGP i S/MIME potpis i šifrovanje; nikakav nalog kod nas, nema piksela za praćenje (udaljene slike isključene dok ih ne dozvoliš po pošiljaocu).

**Zasnovano na:** jezgro Geary-ja za IMAP i SMTP na početku, `lettre` (MIT/Apache) za sastavljanje pošte sa slikama u tekstu, Sequoia-PGP (LGPL) za OpenPGP; interfejs je Neo Kit.

## Razgovor: čet bez naloga [lite]

**Razgovor** (EN: Talk), sa izgledom **WookieChat-a**: teme (Platinum, Fosfor, Prvi sto, krupan retro prozor), klasičan spisak kanala i korisnika, i Dijana u uglu kad ti neko piše.

- **Tekst sa slikama:** GIF, JPG i PNG prikazani u samom razgovoru, i animirani GIF-ovi (mirni kad sistem traži manje pokreta).
- **Fajlovi prevuci-i-pusti**, pravo sa jednog računara na drugi (P2P), bez servera i bez ograničenja veličine osim diska; protokol **LocalSend** (MIT), pa se i telefoni sa LocalSend-om mogu priključiti.
- **Sam nalazi ljude:** računari na istoj mreži vide jedni druge preko mDNS i DNS-SD (Avahi); **bez naloga**, bez broja telefona, samo ime i sličica koju izabereš.
- **Van kućne mreže (opciono):** most ka **XMPP**-u ili **Matrix**-u za prijatelje na drugim mestima, uključuje ga korisnik, a Brava ga prikazuje sa njegovim saobraćajem.
- **Živi fajlovi:** Pero dokument ili DreamCatcher tabla poslata u Razgovoru otvara se kao zajednički dokument koji oboje menjaju.
- **Bezbednost:** poruke na lokalnoj mreži su šifrovane (Noise protokol), ključ drugog računara se prikaže kao kratak spisak reči da ga jednom uporedite; u Dečjoj sobi samo kontakti koje je odobrila odrasla osoba.

## Školska klupa na školskoj mreži [lite]

**Školska klupa** (EN: SchoolDesk) raste od zadataka i rokova ([EVERYDAY-APPS.sr.md](EVERYDAY-APPS.sr.md)) u celu učionicu na školskoj mreži:

- **Učitelj vidi svaki računar u odeljenju** kao pločicu: ko je povezan, koji je zadatak otvoren; klik pokaže ekran jednog učenika, a učenik vidi jasan znak oka kad god se njegov ekran gleda.
- **Emitovanje ekrana:** ekran učitelja (ili rad jednog učenika, uz dozvolu) ide na sve monitore u učionici; Neo + B ga završava.
- **Fajlovi ostaju u školi:** deljenje i skupljanje fajlova ide lokalnom mrežom preko Razgovora, ne preko interneta.
- **DreamCatcher zadaci sa obaveštenjem na stolu:** zadatak od učitelja stiže kao DreamCatcher kartica i tiho obaveštenje na učenikovom stolu (Dijana podigne malu tablu; nikad ne prekida).
- **Zasnovano na:** Veyon (GPL-2.0, otvoreno upravljanje učionicom) za pregled i emitovanje ekrana, Razgovor za fajlove, DreamCatcher za zadatke.

## Da li običan tekst danas ima smisla?

Da, za **brze beleške i kod**: otvara se svuda, mali je i ništa ne može da ga pokvari. Za skoro sve ostalo **Markdown je bolji kompromis**: ispod je i dalje običan tekst, pa traje i svaki program ga čita, ali ima naslove, spiskove, veze i slike. AI modeli prirodno pišu Markdown, a Git može da uporedi dve verzije red po red, što sa DOCX fajlom ne ide. Zato se Pero otvara u Markdown WYSIWYG režimu, običan tekst je jedan klik dalje, a HTML se koristi kad dokumentu treba pravi raspored ili ide kao mejl ili e-knjiga.

## Demo i video

- **HTML demo** `docs/html/sr/demo/neo-pisi-posalji-pricaj.html` (EN: `docs/html/en/demo/neo-write-send-talk.html`): tri režima Pera na jednom dokumentu, Pošalji kao mejl u Pismo, Razgovor sa slikom u tekstu i prevučenim fajlom, i pregled razreda u Školskoj klupi. To je maketa u pregledaču; ništa se ne šalje.
- **Video** (oko jedan minut, SR i EN) snimljen iz demoa: `docs/video/sr/NBPOS-pisi-posalji-pricaj-sr.mp4`, `docs/video/en/NBPOS-write-send-talk-en.mp4`. Oba nose oznaku „plan, još nije kodirano”.

## Koraci kodiranja i procena

Koraci kodiranja 194 do 205 (koraci 160 do 193 su već planirani za Plan 9, programe „zajedno” i Pametno spuštanje), posle Neo Kit-a i sistemskog veb prikaza:

| Korak | Posao |
| --- | --- |
| 194 | Pero: režim običnog teksta i prekidač režima |
| 195 | Pero: Markdown WYSIWYG (pulldown-cmark, oznake u redu sa kursorom) |
| 196 | Pero: HTML WYSIWYG u WPE WebKit-u, podeljen prikaz izvora |
| 197 | Izvoz HTML, PDF i EPUB iz jednog dokumenta |
| 198 | Pismo: IMAP/SMTP, niti, lokalni nacrti, kašnjenje slanja |
| 199 | Pismo: HTML pisanje, šabloni kao Blokovi, pregled priloga |
| 200 | Pošalji kao mejl iz Pera (slike u tekstu, deo u običnom tekstu); pravila i Lux port |
| 201 | Razgovor: mDNS otkrivanje, šifrovan lokalni čet, slike u tekstu, teme |
| 202 | Razgovor: P2P prenos fajlova (LocalSend), prevuci i pusti |
| 203 | Živi fajlovi između Neo korisnika (Pismo i Razgovor, CRDT sloj) |
| 204 | Razgovor: opcioni most ka XMPP-u i Matrix-u |
| 205 | Školska klupa: pregled razreda, emitovanje ekrana, lokalni fajlovi, DreamCatcher zadaci |

Oko **24 čovek-meseca**: režimi i izvoz Pera 6, Pismo 8, Razgovor 6, dodaci Školskoj klupi za učionicu 4 (povrh 5 za Školsku klupu u [EVERYDAY-APPS.sr.md](EVERYDAY-APPS.sr.md)). Licence se ponovo proveravaju pre spajanja koda; Veyon (GPL-2.0) i jezgro Geary-ja (LGPL) rade kao zasebni programi.

Vidi i [FEATURES.sr.md](FEATURES.sr.md), [UNIQUE-FEATURES.sr.md](UNIQUE-FEATURES.sr.md) (stavke 17 i 18), [TOGETHER.sr.md](TOGETHER.sr.md), [APP-NAMES.sr.md](APP-NAMES.sr.md).
