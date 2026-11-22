# JHA Invoice Maker - Version 1.0

## Release Notes (v1.0)
- **Base Version Date**: 22-Nov-2026
- **Full native Rust rewrite** (replaces Electron + Chromium):
  - Single portable `JHA Invoice Maker.exe` of ~13 MB (was ~104 MB), built with `desktop/build.ps1`, download from GitHub Releases.
  - UI rebuilt with egui using the same dark design system, Lexend type, Phosphor icons and layout (New, History, Clients, Settings).
  - Invoice layout engine ported from the invoice CSS. The on-screen preview and the PDF come from the same layout, so they match exactly.
  - Vector PDF export via krilla: single page sized to the invoice, zero margins, selectable text, embedded font subsets, SVG logos stay vector.
  - AES-256 (PDF 2.0, V5/R6) edit protection via lopdf, with the same permissions as v0.8: open, print and copy without a password; editing locked by the owner password.
  - Storage moved from IndexedDB to `%APPDATA%\JHA Invoice Maker\database.json` (atomic writes + `.bak`), or next to the exe in portable mode (`portable.txt`).
  - Automatic one-time import of v0.8 data from the Electron app's local storage. JSON backups stay compatible both ways (schema 2.1, unknown fields preserved).
  - Taka sign (৳) now renders in PDFs via a system-font fallback.
  - First-run demo data (profile, platforms, bank, one client, one invoice) when there is nothing to import.
  - UI polish: one-line client and invoice cards, multi-select bulk delete for clients, pixel-exact icons and preview, 42 px inputs throughout, roomier text areas, compact toggles, consistent heading and label spacing, thousands separators on amounts.
  - Small improvements: Ctrl+S / Ctrl+P shortcuts, Ctrl + mouse-wheel zoom, "Open Folder" for the data file, CSV/JSON export uses the current filters, the WhatsApp line is hidden when no phone number is set, and bulk export reports per-file errors.

---
# JHA Invoice Maker - Version 0.8

## Release Notes (v0.8)
- **Base Version Date**: 09-Nov-2026
- **Features Implemented in v0.8**:
  - **Hardware-Accelerated AES-256 PDF Permissions & Edit Lock Architecture**:
    - Anyone can open, read, and print exported PDFs freely with zero password prompt (`userPassword: ''`).
    - Content copying (`allowCopying: true`) and accessibility extraction (`allowExtraction: true`) are explicitly allowed without requiring any password.
    - Modifying, editing, annotating, form filling, and document assembly in PDF editors (Adobe Acrobat, Foxit, Nitro, etc.) remain strictly locked under the owner password (`ownerPassword: pwd`).
    - Default Save Filename Format: `"Invoice - [Number].pdf"` (e.g. `Invoice - 0034.pdf`) with proper spaces around the hyphen for single exports, 3-dot menu downloads, and bulk exports.
    - Zero-dependency standalone bundled crypto engine (`pdf-crypto.js`) bundled directly into the executable package (`files: [...]`).
    - Live background password synchronization on input and multi-layer fallback lookup (`getActivePdfPassword()`), ensuring export never misses a typed password.
    - Explicit error handling: encryption failures throw immediately and surface a toast rather than silently outputting an unprotected file.
  - **"Delete all data" Complete Reset**:
    - Completely erases all earning histories (`invoices: []`).
    - Completely erases all saved clients (`clients: []`), removing hardcoded sample clients.
    - Resets all issuer profile fields, bank accounts, and payment platforms to empty defaults.
    - Purges all legacy localStorage keys and resets IndexedDB cleanly.
  - **Password Input & Eye Icon Polish**:
    - Removed default browser focus outline ring on the eye button toggle (`outline: none !important; box-shadow: none !important;`).
    - Added clean character letter spacing (`letter-spacing: 0.08em`) when password is in text mode.
    - Added generous bullet letter spacing (`letter-spacing: 0.28em`) in password hidden mode so dots are cleanly spaced.
    - Preserved normal letter spacing for input placeholder text.
    - Inset 30px toggle button with smooth background hover feedback (`rgba(255, 255, 255, 0.08)`).
  - **Modern Sleek Navigation Active Tab Redesign**:
    - Replaced the harsh neon orange top glowing bar and blurred radial bloom with an ultra-clean, modern segmented pill surface design.
    - Active tab features a subtle dark-glass background (`rgba(255, 255, 255, 0.08)`), crisp 1px border (`rgba(255, 255, 255, 0.12)`), pure white text (`#ffffff`, `font-weight: 600`), and a sharp brand orange accent icon without fuzzy glow.
    - Inactive tabs remain flat and muted with subtle hover feedback.
  - **Client Form Label Simplifications**:
    - Renamed `"Notes / Default Terms"` to **`"Note"`**.
    - Renamed `"Phone / WhatsApp"` to **`"Phone"`**.
    - Renamed `"Location / City, Country *"` to **`"Location (City, Country) *"`**.
  - **Dedicated "About Application" Section in Settings**:
    - Added an About App card in the Settings tab positioned before Danger Zone.
    - Consistent Layout & Spacing: Adopted the standard direct section heading (`<h3><i class="ph ph-info"></i> About Application</h3>`) with standard `16px` bottom spacing matching all other sections on the Settings page.
    - Removed redundant top-right version chip badge per user preference.
    - Clean card styling matching `db-hub-card` (border radius 8px, surface background, subtle border).
    - Displays: Creator (`Jarif Hasan Akash`), Current Version (`v0.8 (Stable Desktop)`), Architecture (`Portable Native App - Electron + Chromium`), Storage Engine (`Local IndexedDB + Mirror - Zero Cloud Tracking`), and Security specifications (`Hardware-Accelerated AES-256 Vector PDF Protection & Edit Lock`).
    - Branded icon box with `app-icon.png` (matching desktop `icon.ico` app icon) and desktop edition pill.
  - **Previous Features Retained**:
    - Top header brand cleanup: Removed icon and brand title from top left; moved the 4 navigation tabs (New, History, Clients, Settings) flush to the left.
    - Unified 3-Dot Action Menus for both Invoices and Clients (`Edit`, `Download PDF`, `Duplicate`, `Delete`).
    - Mathematically centered 3-dot vector SVG icon.
    - Custom multi-select checkboxes with perfectly centered vector SVG checkmarks.
    - Floating bottom-centered Bulk Action Bar with count, download, delete, and mathematically centered circular vector SVG close button (`✕`).
    - Subtle neutral invoice number badge (font weight 400).
    - "Showing 1–20 of X invoices" header label on the right side of the "All Invoices" title row.
    - Centered pagination controls with top border removed (20 items per page).
    - Settings page bottom spacing: generous breathing room below Danger Zone.
    - Standalone portable Windows executable (`Invoice Maker.exe`).
    - Safe local versioning workflow (`v0.7` preserved as baseline, `v0.8` active).
