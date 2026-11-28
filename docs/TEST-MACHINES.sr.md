# Probne mašine

Mašine koje projekat poznaje iz njihovih izveštaja. Ovde piše šta Fortress
i Neo DOS danas umeju da kažu o njima; ništa od toga nije obećanje
drajvera. (Ova stranica je o običnim PC računarima. Neo hardver je samo
koncept, vidi [CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md).)

## Threadripper PRO radna stanica sa Intel Arc B580

Iz sistemskog izveštaja koji je poslao probač.

| Deo | Šta je |
|---|---|
| Procesor | AMD Ryzen Threadripper PRO 3945WX (Castle Peak, Zen 2, 12 jezgara, 24 niti, podnožje SP3) |
| Ploča | AMD WRX80 platforma (PCI-Express 4.0) |
| Memorija | 64 GB DDR4 |
| Grafika | Intel Arc B580 Graphics (porodica Battlemage, BMG-G21, ASRock ploča, 12 GB GDDR6, 192 bita, PCI 8086:E20B na magistrali 3, veza x8) |
| Viđeni PCI id-ovi | AMD 1022:1480 korenski kompleks, 1481 IOMMU, 1482 host most, 1483 i 1484 mostovi; Intel 8086:E2F0 i E2FF (mostovi kartice) |

Šta Neo Boing Platinum OS Lite danas radi sa njom:

- **Ekran:** ISO traži od boot loadera 1920 x 1080 sa 32 bita (UEFI GOP,
  koji Arc kartice imaju). Koristi se bilo koji režim koji loader da, a desk ga
  popunjava (do 2560 x 1440; veći ekran dobija desk na sredini, a ostalo je
  crno).
- **Grafički drajver:** još nema. Za Arc kartice u Fortressu nema
  sopstvenog drajvera; ekran je framebuffer boot loadera, bez ubrzanja.
  Sopstveni Arc (Xe2) drajver je ogroman posao i nije u bliskim koracima.
- **Prepoznavanje:** dnevnik pokretanja i `PCI` u Neo DOS-u imenuju AMD
  Zen 2 platformu i kartice Arc B580 i B570, pa se u izveštajima jasno vide
  (`Platform: AMD Starship/Matisse root complex ...`, `Graphics: Intel Arc
  B580 Graphics ... at 03:00.0`).
- **Na ovoj mašini još nema:** NVMe drajvera (jedini drajver diska je
  virtio, pa se PC pokreće sa USB štapa i boot slike, bez diska), više od
  jednog jezgra procesora i zvuka.

Kako se šalje izveštaj: [REAL-PC-TEST.sr.md](REAL-PC-TEST.sr.md).
