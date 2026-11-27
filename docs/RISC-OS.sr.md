# RISC OS ideje na Neo-u

[English](RISC-OS.md) · **Srpski** · Plan, 27. novembar 2026.

RISC OS je oblikovao Traku i „prevuci da sačuvaš” u Neo-u. Ova strana navodi šta je još vredno uzeti. Većina RISC OS izvornog koda je otvorena (Apache 2.0, potvrditi po komponenti), ali ideja je na prvom mestu: Neo uzima ponašanje, ne kod. Stavke **u specifikaciji** već stoje u specifikaciji; **nove** su predlozi koje vlasnik prihvata ili odbacuje.

| # | Ideja | Stanje | Šta Neo dobija |
| --- | --- | --- | --- |
| R1 | Traka sa ikonama i pinboard | u specifikaciji | Traka i Police |
| R2 | Prevuci ikonu iz prozora za čuvanje da sačuvaš | u specifikaciji | Nema lutanja kroz dijalog |
| R3 | **Miš sa tri tastera: Select, Menu, Adjust**. Menu otvara meni ispod pokazivača bilo gde; Adjust radi istu radnju kao Select, ali ostavlja prozor ili meni otvorenim | novo | Uklapa se u opciju „desni taster za menije” iz specifikacije. Adjust je novi deo, na primer otvaranje foldera u novom prozoru bez zatvaranja starog |
| R4 | **Programi kao folderi** (`!App`): dupli klik pokreće, Shift+dupli klik otvara da se vidi unutra | novo | Neo-ovi programi u jednom fajlu zadržavaju veličinu; pogled „otvori program” pokazuje šta sadrži i šta sme |
| R5 | Skripte pri paljenju (Obey fajlovi) | pokriveno | Lux i Neo DOS skripte |
| R6 | BBC BASIC kao prvi jezik | pokriveno, uz pitanje | Neo BASIC je prvi jezik. Proceniti dijalekt BBC BASIC-a (na primer Brandy) za ljude sa starim listinzima |
| R7 | **Pokrenuti sam RISC OS** u Arkadi | novo, za procenu | RPCEmu ili RISC OS na Raspberry Pi ploči za stare RISC OS programe, kao poseban program |
| R8 | Mali ugrađeni programi (Edit, Draw, Paint) sa budžetom veličine | pokriveno | Mali programi sa budžetom memorije, što je već Neo pravilo dizajna |

## Redosled

R3 i R4 su jeftini i stižu uz radnu površinu. R6 i R7 su procene, o kojima se odlučuje posle M2.
