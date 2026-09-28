# JHA Invoice Maker 🧾

> **Privacy-First, Serverless Invoice Maker with 6-Digit End-to-End Encrypted Device Sync & Offline PWA Support.**

![JHA Invoice Maker](jha-icon.png)

A high-performance, browser-native invoice application designed for freelancers, contractors, and small businesses. Generates vector-sharp PDF invoices, stores all data offline in browser storage (IndexedDB), and syncs seamlessly between PC and mobile with zero accounts, zero passwords, and zero tracking.

---

## ✨ Features

- **🔒 100% Client-Side & Private:** All data (clients, invoices, company profiles, bank details) lives exclusively inside your browser's local IndexedDB. No external servers or tracking databases.
- **⚡ 6-Digit E2EE Device Sync:** Effortlessly transfer and merge invoices between your PC and Mobile phone using a private 6-digit pairing code. Encrypted with military-grade PBKDF2 + ChaCha20 / AES-GCM and auto-destructs after 5 minutes.
- **📱 Progressive Web App (PWA):** Installable on iOS (Safari: *Add to Home Screen*) and Android (Chrome: *Install App*). Runs full-screen offline without browser bars.
- **📄 Vector PDF Generation:** High-resolution vector PDF export with dynamic paging, company logo embedding, and multiple international currencies.
- **💾 JSON Database Hub:** One-click full database export and restore. Move your entire accounting history across machines anytime.
- **🌗 Dark Mode UI:** Modern, curated dark aesthetic built for speed and clarity.

---

## 🚀 Live Demo

Open on any PC, Mac, iPhone, or Android device:
**[https://your-username.github.io/invoice-maker/](https://your-username.github.io/invoice-maker/)**

---

## 📲 How to Install as an App (PWA)

### iPhone / iPad (Safari)
1. Open the website in Safari.
2. Tap the **Share** button (box with an arrow pointing up).
3. Scroll down and tap **Add to Home Screen**.
4. Tap **Add**. The app icon will appear on your home screen.

### Android (Chrome)
1. Open the website in Chrome.
2. Tap the **Three Dots (⋮)** in the top right corner.
3. Tap **Install app** or **Add to Home screen**.
4. The standalone app will be added to your app drawer.

### Desktop (Chrome / Edge / Brave)
1. Look at the right side of the address bar.
2. Click the **Install** icon.
3. JHA Invoice Maker will run in its own dedicated desktop window.

---

## 🔄 How 6-Digit Device Sync Works

1. On your **PC**, go to **Settings** $\rightarrow$ Look at the **Device Sync** panel on the left sidebar.
2. Note the 6-digit code (e.g. `582 914`). It is valid for 5 minutes.
3. On your **Phone**, go to **Settings** $\rightarrow$ Type `582 914` into the sync box $\rightarrow$ Tap **Connect & Sync**.
4. Your phone instantly downloads, decrypts, and merges all your invoices and clients.

---

## 🛠️ Technology Stack

- **Frontend:** Vanilla JavaScript (ES6+), HTML5, CSS3 Tokens
- **Icons:** Phosphor Icons
- **PDF Engine:** HTML2PDF.js
- **Storage:** Browser IndexedDB + localStorage mirror
- **Cryptography:** Web Crypto API + Pure-JS PBKDF2, HMAC-SHA256, and ChaCha20 cipher engine
- **PWA:** Web App Manifest + Service Worker offline cache

---

## 📄 License

MIT License. Free to use and customize for personal and commercial projects.
