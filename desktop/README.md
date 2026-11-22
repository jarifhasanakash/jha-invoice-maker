# JHA Invoice Maker — Native Rust Desktop App

A portable Windows app for creating, managing and exporting invoices as
pixel-perfect, single-page vector PDFs. Version 1.0 is a full rewrite in Rust
(previously Electron); the screens, invoice design and data format are the same.

| | Electron (v0.8) | Rust (v1.0) |
|---|---|---|
| Executable | ~104 MB | ~13 MB |
| Runtime | Bundled Chromium | Native (egui + OpenGL) |
| Data | IndexedDB inside Chromium | `database.json` file |

---

## Run

Download `JHA-Invoice-Maker.exe` from the
[latest release](https://github.com/jarifhasanakash/jha-invoice-maker/releases/latest),
or build it yourself (see below) and run `release\JHA Invoice Maker.exe`. No installation is required; copy the
exe anywhere.

**Your data** is stored in `%APPDATA%\JHA Invoice Maker\database.json`
(Settings → JSON Database Engine → *Open Folder*).

* **Portable mode:** put an empty file named `portable.txt` next to the exe and
  the data is kept in `database.json` beside it instead (e.g. on a USB stick).
* **First launch:** the app imports your data from the old Electron version
  automatically. It reads a temporary copy, so the old app's files are never
  changed. JSON backups exported from either version can be restored in the
  other.
* **Brand-new install:** when there's nothing to import, the app starts with
  demo data (a filled-in profile, payment platforms and bank details, one
  client and one invoice) so it doesn't open empty. The PDF password is
  left blank.

---

## Features

* **Invoice editor** with a live preview, zoom (buttons, *Fit*, or Ctrl + mouse
  wheel, which zooms around the pointer), and line items with multi-line sub-details. Optional *Pay in Advance*
  and *Pay upon Completion* rows and an invoice note.
* **Vector PDF export.** Single page, sized to the invoice, no margins,
  selectable text, embedded Lexend fonts. The preview is drawn by the same
  layout engine, so what you see is exactly what gets exported.
* **AES-256 edit protection** (PDF 2.0, V5/R6). Anyone can open, print and copy
  the PDF without a password; editing, annotating, form filling and page
  assembly require the password set in Settings.
* **History & earnings.** Totals, search, status and date filters, 20 per page,
  per-row menus (Edit, Download PDF, Duplicate, Delete), bulk download to a
  folder and bulk delete, and CSV/JSON export of the filtered list.
* **Clients:** search, add/edit with validation, and a per-client currency
  (13 currencies, including ৳ and ₹).
* **Settings:** company logo (PNG, JPG, SVG, WebP or GIF), profile, default
  currency, payment platforms, bank details, JSON backup and restore, and a
  full reset.
* **Shortcuts:** Ctrl+S saves the invoice, Ctrl+P downloads the PDF.

---

## Build

```powershell
.\build.ps1          # -> release\JHA Invoice Maker.exe
.\build.ps1 -Run     # build and start
.\build.ps1 -Test    # run the unit tests
```

This PC has Rust's GNU toolchain but no C compiler or Visual Studio. On first
run, `build.ps1` installs Zig into `.tools\` (needs Python 3) and uses its
`llvm-dlltool` together with Rust's bundled `rust-lld` linker.

Alternatively, install *Visual Studio Build Tools* (C++ workload) and run
`rustup default stable-x86_64-pc-windows-msvc`. A plain
`cargo build --release` then works.

Windows resources (icon, version info, manifest) are in `assets\app.rc` →
`assets\app.res`. Rebuild with `zig rc /fo app.res app.rc` after editing.

### Source layout

| Path | Purpose |
|---|---|
| `src/model.rs` | Database schema, migrations, import of current and legacy backups, CSV |
| `src/storage.rs` | `database.json` load/save (atomic, with `.bak`), import from the Electron version |
| `src/layout.rs` | Invoice layout engine (port of the original invoice CSS) |
| `src/fonts.rs` | Font loading, shaping (rustybuzz), system fallback for ৳ |
| `src/render.rs` | Raster preview renderer (tiny-skia) |
| `src/pdf.rs` | PDF writer (krilla) and AES-256 protection (lopdf) |
| `src/ui/` | egui interface: editor, history, clients, settings, widgets, theme |
| `assets/` | Lexend fonts (static weights), Phosphor icons, app icon, Windows resources |

The web version (`index.html`, `script.js`, `styles.css`, …) lives in the
repository root and is published with GitHub Pages. The Rust app does not use it.
