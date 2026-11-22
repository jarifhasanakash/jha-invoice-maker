# JHA Invoice Maker 🧾

> **Privacy-first invoice maker for the browser and Windows, with a Rust PDF engine.**

![JHA Invoice Maker](jha-icon.png)

An invoice app for freelancers, contractors and small businesses. It creates single-page vector PDF invoices and keeps all data on your device (browser storage on the web, a local file on Windows). No accounts, no servers, no tracking.

---

## ✨ Features

- **🔒 Private:** clients, invoices, profile and bank details stay on your device. Nothing is sent to a server.
- **📄 Vector PDF engine in Rust:** the web version and the Windows app share the same Rust PDF engine (compiled to WebAssembly for the browser). Single page sized to the invoice, selectable text, embedded fonts, vector logo.
- **🛡️ AES-256 edit protection:** set a PDF password in Settings. Anyone can open, print and copy the PDF; editing requires the password.
- **📱 Works on phones and tablets:** responsive layout with the Save / Download PDF actions pinned to the bottom of the screen.
- **💾 JSON backup:** one-click full export and restore, compatible between the web version and the Windows app.
- **💱 13 currencies,** including ৳ and ₹.

---

## 🚀 Live Demo

Open on any PC, Mac, iPhone, or Android device:
**[https://jarifhasanakash.github.io/jha-invoice-maker/](https://jarifhasanakash.github.io/jha-invoice-maker/)**

---

## 🖥️ Windows Desktop App (v1.0, native Rust)

A portable Windows app with the same screens and invoice design, rewritten in Rust. No installer and no browser needed.

**[⬇️ Download JHA Invoice Maker.exe](https://github.com/jarifhasanakash/jha-invoice-maker/releases/latest/download/JHA-Invoice-Maker.exe)** (~13 MB, Windows 10/11)

- Single ~13 MB executable (the previous Electron build was ~104 MB).
- Vector PDF export with selectable text and optional AES-256 edit protection.
- Data is kept in `%APPDATA%\JHA Invoice Maker\database.json`, or next to the exe in portable mode.
- JSON backups are compatible with the web version in both directions.

Source code and build instructions: [`desktop/`](desktop/).

---

## 🛠️ Technology Stack

- **Web UI:** Vanilla JavaScript, HTML5, CSS
- **PDF engine:** Rust (krilla for vector PDF, lopdf for AES-256), compiled to WebAssembly: [`web-engine/`](web-engine/) builds `pdf-engine.wasm`
- **Desktop app:** Rust, egui, sharing the same layout and PDF code: [`desktop/`](desktop/)
- **Icons:** Phosphor Icons
- **Fonts:** Lexend; Noto Sans Bengali for the Taka sign (SIL Open Font License)
- **Storage:** Browser IndexedDB + localStorage mirror (web), `database.json` (Windows)

### Rebuilding the PDF engine

```powershell
rustup target add wasm32-unknown-unknown
.\web-engine\build.ps1     # -> pdf-engine.wasm
```

---

## 📄 License

MIT License. Free to use and customize for personal and commercial projects.
