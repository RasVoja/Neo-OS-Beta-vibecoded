# Kodiranje dok čekaš sledeću sesiju: tvoj PC radi po zadatku

[English](SCHEDULED-CODING.md) · **Srpski** · 27. novembar 2026.

Cilj: tvoj Threadripper PRO 3945WX (Windows) po rasporedu pokreće Claude Code, koji uradi **jedan korak** iz reda čekanja ([HANDOFF.sr.md](HANDOFF.sr.md)), proveri probe paljenja i otvori pull request. Ti ujutru pregledaš. Ništa ne ide na `main`.

## Šta ti treba

- Claude nalog sa Claude Code pristupom: Pro, Max, Team, Enterprise ili Console. Besplatan claude.ai plan **ne** uključuje Claude Code ([setup](https://code.claude.com/docs/en/setup.md)).
- WSL2 sa Ubuntuom. Claude Code radi i na samom Windowsu, ali ga preporučujem u WSL2 jer su Rust, QEMU i GRUB alati za Linux, a WSL2 podržava i sandbox ([setup](https://code.claude.com/docs/en/setup.md)).
- GitHub nalog sa pravom upisa u repo.
- Računar uključen (ili probuđen zadatkom) i prijavljen u vreme izvršavanja.

## Koraci

1. **WSL2** (PowerShell kao administrator, jednom): `wsl --install -d Ubuntu`, restart, napravi korisnika.
2. **Unutar Ubuntua:** `git clone https://github.com/RasVoja/Neo-OS-Beta-vibecoded ~/nbpos`, zatim `bash ~/nbpos/scripts/setup-wsl.sh` (instalira QEMU, GRUB, xorriso, OVMF, pandoc, Rust sa ciljevima, `gh` i Claude Code).
3. **Prijave, jednom i ručno:** `gh auth login`, pa `claude` (prijava preko pregledača), pa `/exit`.
4. **Ručna proba:** `cd ~/nbpos && ./scripts/test-boot-x86_64.sh`. Dok ovo ne prolazi, ne automatizuj.
5. **Suva proba:** `bash scripts/nightly-step.sh --dry-run`.
6. **Prvi pravi korak, uz tebe:** `bash scripts/nightly-step.sh`. Pogledaj PR.
7. **Zakazivanje:** PowerShell: `powershell -ExecutionPolicy Bypass -File \\wsl$\Ubuntu\home\<ti>\nbpos\scripts\register-task.ps1` (vreme i dani su parametri). Probaj sa `Start-ScheduledTask -TaskName 'NBPOS nightly step'`.

## Kako je ograničeno

`scripts/nightly-step.sh` poziva `claude -p` (neinteraktivno, [docs](https://code.claude.com/docs/en/headless.md)) sa:

| Opcija | Zašto |
| --- | --- |
| `--permission-mode acceptEdits` | menja fajlove bez pitanja |
| `--allowedTools "…"` | samo navedene alatke: git (push samo na `claude/*`), `gh pr create`, `cargo`, skripte iz repoa. Sve drugo što bi tražilo odobrenje se odbija |
| `--permission-prompts none` | niko nije tu da odgovori; traži Claude Code 2.1.259 ili noviji |
| `--max-budget-usd` | gornja granica troška po pokretanju (podrazumevano 15 USD, menja se sa `NEO_NIGHT_BUDGET_USD`) |
| `timeout 4h` | gornja granica vremena |
| `flock` | nikad dva pokretanja istovremeno |

Pravila za sam korak su u [scripts/nightly-prompt.md](../scripts/nightly-prompt.md): jedna grana `claude/neo-lite-stepNN-…`, probe paljenja pre i posle, bez force-push, bez `main`, PROGRESS i STATUS, PR. Ako neka proba padne, korak se vraća i ništa se ne šalje.

**Ono što skripta sama ne garantuje:** obrazac `git push origin claude/*` ne zabranjuje izričito `--force`. Zato u GitHub-u uključi zaštitu grane za `main` (zabrana direktnog pisanja i force-push), a PR-ove spaja samo ti.

## Troškovi i sigurnost

- Pokretanja troše tvoje Claude ograničenje ili Console kredit; isprati prvo nekoliko pokretanja. `last-summary.txt` u `~/nbpos-night-logs/` ima trošak (procena klijenta, ne račun).
- Claude dobija pravo upisa na tvoj repo preko `gh` prijave. Koristi nalog/token sa pristupom samo ovom repou.
- Pokreće se kod koji Claude napiše, na tvom računaru (QEMU, cargo build). Radi u WSL2, ne na Windowsu, i držiš ga u kućnom folderu Ubuntua.
- Pregledaj svaki PR pre spajanja.

## Ako ne radi

- `claude: command not found` u zadatku: Task Scheduler koristi `bash -lc`; proveri da je `~/.local/bin` u PATH-u (skripta ga sama dodaje).
- Nema `/dev/kvm`: QEMU radi sporije, probe traju duže; povećaj `NEO_NIGHT_MAX_HOURS`.
- Prijava istekla: ponovi `claude` ručno. Za potpuno bezlično okruženje možeš koristiti Console API ključ (`ANTHROPIC_API_KEY`), koji se naplaćuje po upotrebi.
- Laptop/PC spava: `-WakeToRun` je uključen, ali BIOS i plan napajanja moraju to da dozvole.

## Šta nisam mogao da probam

Ni skripte ni zadatak nisu pokrenuti na tvom računaru (ovde nema WSL-a, QEMU-a ni Rust ciljeva). Opcije su proverene u dokumentaciji, ali prvo pokretanje radi uz nadzor.
