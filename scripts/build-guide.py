#!/usr/bin/env python3
"""Builds the user guide in guide/ from the Markdown pages in docs/.

    python3 scripts/build-guide.py          # HTML for both languages, then PDFs
    python3 scripts/build-guide.py --no-pdf # HTML only

Output: guide/html-en/*.html, guide/html-sr/*.sr.html, guide/NBPOS-guide-en.pdf,
guide/NBPOS-vodic-sr.pdf. Needs pandoc. For the PDFs it uses wkhtmltopdf if it is
installed, otherwise a headless Chromium or Chrome (set CHROME=/path to choose one).
"""
import glob, html, os, re, shutil, subprocess, sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DOCS = os.path.join(ROOT, "docs")
GUIDE = os.path.join(ROOT, "guide")
SKIP = {"PROGRESS"}  # the developer log stays in docs/ only

# Pages shown first on the index, in this order; the rest follow alphabetically.
FIRST = ["HELP", "FEATURES", "UNIQUE-FEATURES", "MULTI-MONITOR", "STATUS", "BUILD-ISO", "REAL-PC-TEST", "GAMES", "CREATIVE-APPS", "MUSIC",
         "KIDS", "APP-NAMES", "DREAMCATCHER", "TOGETHER", "EVERYDAY-APPS", "PLAN9", "RAD", "MASCOT", "RESIDENTS"]
TEXT = {
    "en": dict(title="Neo Boing Platinum OS: user guide", other="Srpski", pdf="NBPOS-guide-en.pdf",
               intro="Every plan and feature page of Neo Boing Platinum OS, the system for generic x86_64 and ARM computers. "
                     "These pages open from a folder or a USB stick, with no server and no network.",
               sys="The system", concept="Separately: the ideal computer concept (concept hardware)",
               more="More", readme="Project page (README)", demos="System demos in the browser",
               manual="Beta manual of the desktop", whole="The whole guide as one PDF"),
    "sr": dict(title="Neo Boing Platinum OS: korisnički vodič", other="English", pdf="NBPOS-vodic-sr.pdf",
               intro="Sve strane planova i mogućnosti Neo Boing Platinum OS-a, sistema za obične x86_64 i ARM računare. "
                     "Strane se otvaraju iz foldera ili sa USB ključa, bez servera i bez mreže.",
               sys="Sistem", concept="Odvojeno: koncept idealnog računara (konceptni hardver)",
               more="Još", readme="Strana projekta (README)", demos="Demoi sistema u pregledaču",
               manual="Beta priručnik radne površine", whole="Ceo vodič kao jedan PDF"),
}
DEMO = {"en": "../../docs/html/en/demo/neo-demos.html", "sr": "../../docs/html/sr/demo/neo-demoi.html"}
MANUAL = {"en": "../../docs/html/en/neo-manual-beta.html", "sr": "../../docs/html/sr/neo-prirucnik-beta.html"}


def pages(lang):
    if lang == "sr":
        files = glob.glob(os.path.join(DOCS, "*.sr.md"))
    else:
        files = [f for f in glob.glob(os.path.join(DOCS, "*.md")) if not f.endswith(".sr.md")]
    names = {os.path.basename(f)[:-3]: f for f in files}
    names = {k: v for k, v in names.items() if k.replace(".sr", "") not in SKIP}
    key = lambda n: (FIRST.index(n.replace(".sr", "")) if n.replace(".sr", "") in FIRST else 99, n)
    return sorted(names.items(), key=lambda kv: key(kv[0]))


def title_of(md):
    for line in open(md, encoding="utf-8"):
        if line.startswith("# "):
            return line[2:].strip()
    return os.path.basename(md)


def fix_links(text, lang):
    """Rewrite links and image sources that were written relative to docs/ so they work from
    guide/html-<lang>/."""
    other = "sr" if lang == "en" else "en"

    def one(m):
        attr, href = m.group(1), m.group(2)
        if re.match(r"^(https?:|mailto:|#)", href):
            return m.group(0)
        path, _, frag = href.partition("#")
        frag = "#" + frag if frag else ""
        if path.startswith("../guide/"):
            new = "../" + path[len("../guide/"):]
        elif path.startswith("html/") or path.startswith("img/") or path.startswith("pdf/") or path.startswith("assets/"):
            new = "../../docs/" + path
        elif path.startswith("../"):
            new = "../" + path  # repository root
            if new.endswith(".md"):
                new = new  # README, CONTRIBUTING stay Markdown
        elif re.match(r"^[A-Z0-9-]+(\.sr)?\.md$", path):
            base = path[:-3]
            if base.replace(".sr", "") in SKIP or not os.path.exists(os.path.join(DOCS, path)):
                new = "../../docs/" + path
            else:
                is_sr = base.endswith(".sr")
                target = "sr" if is_sr else "en"
                new = base + ".html" if target == lang else "../html-%s/%s.html" % (other, base)
        elif path.endswith(".md") or "/" in path:
            new = "../../docs/" + path
        else:
            new = path
        return '%s="%s%s"' % (attr, new, frag)

    return re.sub(r'(href|src)="([^"]+)"', one, text)


def build_lang(lang):
    t = TEXT[lang]
    out = os.path.join(GUIDE, "html-" + lang)
    os.makedirs(out, exist_ok=True)
    for old in glob.glob(os.path.join(out, "*.html")):
        os.remove(old)
    shutil.copy(os.path.join(DOCS, "assets", "guide.css"), os.path.join(out, "guide.css"))
    items = pages(lang)
    for name, md in items:
        res = subprocess.run(["pandoc", md, "-f", "gfm", "-t", "html5", "-s", "--metadata",
                              "pagetitle=" + title_of(md), "-c", "guide.css"],
                             check=True, capture_output=True, text=True).stdout
        res = fix_links(res, lang)
        back = '<p class="guide-nav"><a href="index.html">%s</a></p>\n' % html.escape(t["title"])
        res = res.replace("<body>\n", "<body>\n" + back, 1)
        open(os.path.join(out, name + ".html"), "w", encoding="utf-8").write(res)

    concept = [kv for kv in items if kv[0].replace(".sr", "") == "CONCEPT-COMPUTER"]
    system = [kv for kv in items if kv not in concept]
    li = lambda kv: '<li><a href="%s.html">%s</a></li>' % (kv[0], html.escape(title_of(kv[1])))
    other = "sr" if lang == "en" else "en"
    idx = ['<!doctype html><html lang="%s"><meta charset="utf-8">' % lang,
           '<meta name="viewport" content="width=device-width, initial-scale=1">',
           '<title>%s</title><link rel="stylesheet" href="guide.css"><body>' % html.escape(t["title"]),
           '<p><a href="../html-%s/index.html">%s</a></p>' % (other, t["other"]),
           "<h1>%s</h1><p>%s</p>" % (html.escape(t["title"]), html.escape(t["intro"])),
           "<h2>%s</h2><ul>" % t["sys"], *[li(kv) for kv in system], "</ul>",
           "<h2>%s</h2><ul>" % t["more"],
           '<li><a href="../%s">%s</a></li>' % (t["pdf"], t["whole"]),
           '<li><a href="%s">%s</a></li>' % (MANUAL[lang], t["manual"]),
           '<li><a href="%s">%s</a></li>' % (DEMO[lang], t["demos"]),
           '<li><a href="../../%s">%s</a></li>' % ("README.md" if lang == "en" else "README.sr.md", t["readme"]),
           "</ul>", "<h2>%s</h2><ul>" % t["concept"], *[li(kv) for kv in concept], "</ul></body></html>"]
    open(os.path.join(out, "index.html"), "w", encoding="utf-8").write("\n".join(idx) + "\n")
    return items


def build_pdf(lang, items):
    t = TEXT[lang]
    out = os.path.join(GUIDE, "html-" + lang)
    book = os.path.join(out, "_book.html")
    subprocess.run(["pandoc", *[md for _, md in items], "-f", "gfm", "-t", "html5", "-s", "--toc",
                    "--metadata", "title=" + t["title"], "-c", "guide.css", "-o", book], check=True)
    pdf = os.path.join(GUIDE, t["pdf"])
    try:
        if shutil.which("wkhtmltopdf"):
            subprocess.run(["wkhtmltopdf", "--enable-local-file-access", "-q", book, pdf], check=True)
        else:
            chrome = os.environ.get("CHROME") or next((c for c in ["/opt/pw-browsers/chromium/chrome-linux/chrome",
                     shutil.which("chromium") or "", shutil.which("chromium-browser") or "",
                     shutil.which("google-chrome") or ""] if c and os.path.exists(c)), None)
            if not chrome:
                print("no wkhtmltopdf or Chrome found; skipping PDF for", lang)
                return
            subprocess.run([chrome, "--headless", "--no-sandbox", "--disable-gpu", "--no-pdf-header-footer",
                            "--print-to-pdf=" + pdf, "file://" + book], check=True, capture_output=True)
        print(pdf)
    finally:
        if os.path.exists(book):
            os.remove(book)


if __name__ == "__main__":
    for lang in ("en", "sr"):
        items = build_lang(lang)
        print(os.path.join(GUIDE, "html-" + lang), len(items), "pages")
        if "--no-pdf" not in sys.argv:
            build_pdf(lang, items)
