# Bluetooth u NBPOS Lite

[English](BLUETOOTH.md) · **Srpski** · Ažurirano 28. novembra 2026

## Gde smo (korak 120, početak)

- USB Bluetooth adapteri (klasa e0.01.01, npr. Intel 8087:0032 na mnogim PC-jevima) se **nalaze i imenuju** u komandi `USB` i u boot logu: "Bluetooth adapter (not driven yet)".
- Još ništa ne priča sa njima. Bežični miševi i tastature u Bluetooth režimu još ne rade; u režimu sa 2,4 GHz ključićem (Razer, Logitech) to su obični USB uređaji i rade kao takvi.

## Plan

1. HCI preko USB-a (komande na endpoint-u 0, događaji i ACL podaci na interrupt i bulk endpoint-ima): reset, čitanje adrese, skeniranje.
2. HID preko GATT-a i klasični HID: uparivanje tastature i miša (prvi cilj: korisnikovi Razer uređaji u Bluetooth režimu).
3. Bluetooth panel na desktopu (lista, uparivanje, zaboravljanje), uz Dijaninu pomoć.
4. Kasnije: zvuk (A2DP) i prenos fajlova.

Oznaka: **[next]** za korak 1, **[lite]** za ostalo.
