# Početni zadaci

*English: [STARTER-TASKS.md](STARTER-TASKS.md)*

Dvanaest zadataka za prvi doprinos. Svaki je mali, dira jedno ili dva
mesta i završava se testom koji skripte za podizanje mogu da provere.
Postaće issue-i na javnom repozitorijumu onog dana kad se kod preseli
tamo. Prvo pročitaj [CONTRIBUTING.sr.md](../CONTRIBUTING.sr.md): građenje,
testiranje i pravila koja izmena mora da poštuje.

Veličine: **S** je jedno veče, **M** jedan vikend.

## Neo DOS

1. **Komande `TIME` i `DATE`** (S). Neo DOS nema sat. Daj konzoli način da
   ga pročita (servis za ekran već čita CMOS sat na PC-ju i PL031 na
   AArch64; vidi `user/display`) ili dodaj mali sistemski poziv koji vraća
   vreme. *Test:* serijska sesija u obe skripte kuca `time` i očekuje red
   oblika `[0-2][0-9]:[0-5][0-9]`.
2. **Komanda `MEM`** (S). Prikazuje slobodnu memoriju. Treba novi
   sistemski poziv (vidi „Dodavanje sistemskog poziva” u CONTRIBUTING-u)
   koji vraća broj slobodnih okvira. *Test:* `mem` ispisuje
   `Free memory: N KiB`, gde je N veće od 100000, na obe arhitekture.
3. **Dopunjavanje imena fajlova tasterom Tab** (M). U `read_line`, Tab
   dopunjava reč pre kursora imenima fajlova sa tekućeg volumena
   (`neo_rt::list`). *Test:* `type REA`, pa Tab i Enter, prikazuje README.
4. **Ctrl+A, Ctrl+E, Ctrl+U** (S). Početak reda, kraj reda, brisanje
   reda, kao u većini školjki. `neo-keys` prvo mora da pretvori Ctrl+slovo
   u bajtove od 1 do 26. *Test:* serijska sesija šalje `\001`, `\005` i
   `\025` i proverava ispisan rezultat.
5. **`DIR` sortiran po imenu, sa poravnatim veličinama** (S). *Test:* prvi
   redovi `dir`-a na BOOT: idu abecednim redom.

## Tastatura i tekst

6. **Srpska ćirilica** (M). Dodaj ćirilična slova u font
   (`scripts/make-font.py`, pa ponovo napravi `user/neo-gfx/src/font.rs`)
   i treći raspored u `user/neo-keys`, koji Alt+Shift bira redom. *Test:*
   PS/2 samoprovera kuca ж, a Traka pokazuje `СР`.
7. **Lampica za Caps Lock** (S). PS/2 servis za tastaturu šalje komandu
   0xED sa bitovima lampica kad se Caps Lock promeni. *Test:* samoprovera
   proverava da je kontroler potvrdio (0xFA) oba bajta.

## Radna površina i prozori

8. **Dugme za zatvaranje** (S). Narandžasta tačka na naslovnoj traci
   programa šalje tom programu `EVENT_QUIT`. *Test:* u x86 testu klikova,
   klik na tačku završava `hello-window` njegovim kodom za izlaz.
9. **Prozori na Traci** (M). Svaki prozor programa dobija dugme na Traci
   sa svojim naslovom; klik na njega podiže prozor. *Test:* dok je
   `hello-window` otvoren, piksel njegovog dugmeta na Traci je beo, a klik
   na dugme podiže prozor iznad Neo DOS-a.
10. **Datum na pločici Sat** (S). Prikaži dan i mesec ispod vremena.
    *Test:* samoprovera ekrana proverava da je piksel u redu sa datumom
    boje mastila.

## Jezgro i testovi

11. **Još jedan test za pogrešan handle** (S). Kroz program pozovi
    `send_handle` sa handle-om koji ne postoji i sa handle-om pogrešne
    vrste; očekuj `ERR_BAD_HANDLE` i `ERR_DENIED`. *Test:* nov red u
    samoproveri Rust programa.
12. **Prsten u oba smera** (M). Par prstenova između dva programa (servis
    koji vraća primljeno): ono što uđe u jedan vraća se kroz drugi.
    *Test:* samoprovera šalje 256 KiB, dobija iste bajtove nazad i ništa
    ne curi.
