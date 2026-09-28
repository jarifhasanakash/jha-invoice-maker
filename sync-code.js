/**
 * JHA Invoice Maker - 6-Digit End-to-End Encrypted Device Sync (sync-code.js)
 * 100% Free, Zero-Server Cost, Zero Camera Access, Zero Accounts.
 * Ephemeral 5-Minute Pairing with Native Web Crypto AES-GCM (256-bit).
 */

const SyncCode = (() => {
    const RELAY_BASE = 'https://ntfy.sh';
    const SALT = new TextEncoder().encode('JHA_E2EE_SYNC_2026');
    const CODE_EXPIRY_SECONDS = 300; // 5 minutes

    let activeSyncCode = null;
    let timerInterval = null;
    let secondsRemaining = 0;
    let isSending = false;
    let isReceiving = false;

    /* ---------------- End-to-End Encryption Engine (Universal) ---------------- */

    // SHA-256 Round Constants
    const SHA256_K = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    ];

    /**
     * Pure JavaScript SHA-256 implementation (works in insecure context / HTTP on mobile)
     */
    const jsSha256 = (bytes) => {
        if (typeof bytes === 'string') bytes = new TextEncoder().encode(bytes);
        const l = bytes.length * 8;
        const n = ((bytes.length + 8) >> 6) + 1;
        const words = new Uint32Array(n * 16);
        for (let i = 0; i < bytes.length; i++) words[i >> 2] |= bytes[i] << (24 - (i % 4) * 8);
        words[bytes.length >> 2] |= 0x80 << (24 - (bytes.length % 4) * 8);
        words[words.length - 1] = l;
        let h0 = 0x6a09e667, h1 = 0xbb67ae85, h2 = 0x3c6ef372, h3 = 0xa54ff53a;
        let h4 = 0x510e527f, h5 = 0x9b05688c, h6 = 0x1f83d9ab, h7 = 0x5be0cd19;
        const w = new Uint32Array(64);
        for (let i = 0; i < words.length; i += 16) {
            for (let j = 0; j < 16; j++) w[j] = words[i + j];
            for (let j = 16; j < 64; j++) {
                const s0 = ((w[j - 15] >>> 7) | (w[j - 15] << 25)) ^ ((w[j - 15] >>> 18) | (w[j - 15] << 14)) ^ (w[j - 15] >>> 3);
                const s1 = ((w[j - 2] >>> 17) | (w[j - 2] << 15)) ^ ((w[j - 2] >>> 19) | (w[j - 2] << 13)) ^ (w[j - 2] >>> 10);
                w[j] = (w[j - 16] + s0 + w[j - 7] + s1) | 0;
            }
            let a = h0, b = h1, c = h2, d = h3, e = h4, f = h5, g = h6, h = h7;
            for (let j = 0; j < 64; j++) {
                const S1 = ((e >>> 6) | (e << 26)) ^ ((e >>> 11) | (e << 21)) ^ ((e >>> 25) | (e << 7));
                const ch = (e & f) ^ ((~e) & g);
                const temp1 = (h + S1 + ch + SHA256_K[j] + w[j]) | 0;
                const S0 = ((a >>> 2) | (a << 30)) ^ ((a >>> 13) | (a << 19)) ^ ((a >>> 22) | (a << 10));
                const maj = (a & b) ^ (a & c) ^ (b & c);
                const temp2 = (S0 + maj) | 0;
                h = g; g = f; f = e; e = (d + temp1) | 0;
                d = c; c = b; b = a; a = (temp1 + temp2) | 0;
            }
            h0 = (h0 + a) | 0; h1 = (h1 + b) | 0; h2 = (h2 + c) | 0; h3 = (h3 + d) | 0;
            h4 = (h4 + e) | 0; h5 = (h5 + f) | 0; h6 = (h6 + g) | 0; h7 = (h7 + h) | 0;
        }
        const out = new Uint8Array(32);
        const view = new DataView(out.buffer);
        [h0, h1, h2, h3, h4, h5, h6, h7].forEach((val, idx) => view.setUint32(idx * 4, val, false));
        return out;
    };

    /**
     * Compute SHA-256 hash as hex string with subtle crypto or JS fallback
     */
    const sha256Hex = async (str) => {
        try {
            if (window.crypto && window.crypto.subtle && window.crypto.subtle.digest) {
                const buf = new TextEncoder().encode(str);
                const hashBuf = await crypto.subtle.digest('SHA-256', buf);
                const hashArr = Array.from(new Uint8Array(hashBuf));
                return hashArr.map(b => b.toString(16).padStart(2, '0')).join('');
            }
        } catch (e) {
            // fallback
        }
        const bytes = jsSha256(str);
        return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('');
    };

    /**
     * Pure JS HMAC-SHA256
     */
    const jsHmacSha256 = (key, msg) => {
        if (typeof key === 'string') key = new TextEncoder().encode(key);
        if (typeof msg === 'string') msg = new TextEncoder().encode(msg);
        let k = key;
        if (k.length > 64) k = jsSha256(k);
        const paddedK = new Uint8Array(64);
        paddedK.set(k);
        const iPad = new Uint8Array(64);
        const oPad = new Uint8Array(64);
        for (let i = 0; i < 64; i++) {
            iPad[i] = paddedK[i] ^ 0x36;
            oPad[i] = paddedK[i] ^ 0x5c;
        }
        const inner = new Uint8Array(64 + msg.length);
        inner.set(iPad);
        inner.set(msg, 64);
        const innerHash = jsSha256(inner);
        const outer = new Uint8Array(64 + 32);
        outer.set(oPad);
        outer.set(innerHash, 64);
        return jsSha256(outer);
    };

    /**
     * Pure JS PBKDF2 with HMAC-SHA256 (5000 iterations for high security + speed on mobile)
     */
    const jsPbkdf2 = (password, salt, iterations, length) => {
        const pwd = typeof password === 'string' ? new TextEncoder().encode(password) : password;
        const s = typeof salt === 'string' ? new TextEncoder().encode(salt) : salt;
        const blocks = Math.ceil(length / 32);
        const result = new Uint8Array(blocks * 32);
        for (let b = 1; b <= blocks; b++) {
            const initial = new Uint8Array(s.length + 4);
            initial.set(s);
            initial[s.length] = (b >> 24) & 0xff;
            initial[s.length + 1] = (b >> 16) & 0xff;
            initial[s.length + 2] = (b >> 8) & 0xff;
            initial[s.length + 3] = b & 0xff;
            let u = jsHmacSha256(pwd, initial);
            const t = new Uint8Array(u);
            for (let iter = 1; iter < iterations; iter++) {
                u = jsHmacSha256(pwd, u);
                for (let i = 0; i < 32; i++) t[i] ^= u[i];
            }
            result.set(t, (b - 1) * 32);
        }
        return result.slice(0, length);
    };

    /**
     * Pure JS ChaCha20 stream cipher (RFC 8439)
     */
    const jsChaCha20 = (key, nonce, counter, data) => {
        const state = new Uint32Array(16);
        state[0] = 0x61707865; state[1] = 0x3320646e; state[2] = 0x79622d32; state[3] = 0x6b206574;
        const kView = new DataView(key.buffer, key.byteOffset, key.byteLength);
        for (let i = 0; i < 8; i++) state[4 + i] = kView.getUint32(i * 4, true);
        state[12] = counter;
        const nView = new DataView(nonce.buffer, nonce.byteOffset, nonce.byteLength);
        state[13] = nView.getUint32(0, true);
        state[14] = nView.getUint32(4, true);
        state[15] = nView.getUint32(8, true);

        const out = new Uint8Array(data.length);
        const block = new Uint8Array(64);
        const bView = new DataView(block.buffer);

        for (let pos = 0; pos < data.length; pos += 64) {
            state[12] = counter + Math.floor(pos / 64);
            const working = new Uint32Array(state);
            const rotl = (v, c) => (v << c) | (v >>> (32 - c));
            const qRound = (a, b, c, d) => {
                working[a] = (working[a] + working[b]) | 0; working[d] = rotl(working[d] ^ working[a], 16);
                working[c] = (working[c] + working[d]) | 0; working[b] = rotl(working[b] ^ working[c], 12);
                working[a] = (working[a] + working[b]) | 0; working[d] = rotl(working[d] ^ working[a], 8);
                working[c] = (working[c] + working[d]) | 0; working[b] = rotl(working[b] ^ working[c], 7);
            };
            for (let round = 0; round < 10; round++) {
                qRound(0, 4, 8, 12); qRound(1, 5, 9, 13); qRound(2, 6, 10, 14); qRound(3, 7, 11, 15);
                qRound(0, 5, 10, 15); qRound(1, 6, 11, 12); qRound(2, 7, 8, 13); qRound(3, 4, 9, 14);
            }
            for (let i = 0; i < 16; i++) {
                bView.setUint32(i * 4, (working[i] + state[i]) | 0, true);
            }
            const len = Math.min(64, data.length - pos);
            for (let i = 0; i < len; i++) {
                out[pos + i] = data[pos + i] ^ block[i];
            }
        }
        return out;
    };

    /**
     * Get cryptographically random bytes with fallback
     */
    const getRandomBytes = (len) => {
        const bytes = new Uint8Array(len);
        if (typeof window !== 'undefined' && window.crypto && window.crypto.getRandomValues) {
            window.crypto.getRandomValues(bytes);
        } else {
            for (let i = 0; i < len; i++) {
                bytes[i] = Math.floor(Math.random() * 256);
            }
        }
        return bytes;
    };

    /**
     * Compress string using gzip if supported
     */
    const compressText = async (text) => {
        if (typeof CompressionStream !== 'undefined') {
            try {
                const stream = new Blob([text]).stream();
                const compressedStream = stream.pipeThrough(new CompressionStream('gzip'));
                const buffer = await new Response(compressedStream).arrayBuffer();
                return new Uint8Array(buffer);
            } catch (e) {
                console.warn('[SyncCode] Compression failed, falling back to raw:', e);
            }
        }
        return new TextEncoder().encode(text);
    };

    /**
     * Decompress binary to string using gzip
     */
    const decompressBytes = async (bytes, isCompressed) => {
        if (isCompressed && typeof DecompressionStream !== 'undefined') {
            try {
                const stream = new Blob([bytes]).stream();
                const decompressedStream = stream.pipeThrough(new DecompressionStream('gzip'));
                return await new Response(decompressedStream).text();
            } catch (e) {
                console.warn('[SyncCode] Decompression failed, falling back to raw decode:', e);
            }
        }
        return new TextDecoder().decode(bytes);
    };

    /**
     * Convert Uint8Array to base64
     */
    const bytesToBase64 = (bytes) => {
        let binary = '';
        for (let i = 0; i < bytes.byteLength; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        return btoa(binary);
    };

    /**
     * Convert base64 to Uint8Array
     */
    const base64ToBytes = (base64) => {
        const binary = atob(base64);
        const bytes = new Uint8Array(binary.length);
        for (let i = 0; i < binary.length; i++) {
            bytes[i] = binary.charCodeAt(i);
        }
        return bytes;
    };

    /**
     * Encrypt data package with 6-digit PIN using authenticated cipher
     */
    const encryptPackage = async (dataObj, pin) => {
        const jsonStr = JSON.stringify(dataObj);
        const compressed = await compressText(jsonStr);
        const isGzip = typeof CompressionStream !== 'undefined';

        // Derive 64-byte key material: 32 bytes encryption key + 32 bytes HMAC auth key
        const derived = jsPbkdf2(pin, SALT, 5000, 64);
        const encKey = derived.slice(0, 32);
        const authKey = derived.slice(32, 64);

        const nonce = getRandomBytes(12);
        const ciphertext = jsChaCha20(encKey, nonce, 1, compressed);
        const mac = jsHmacSha256(authKey, ciphertext);

        return {
            v: 2,
            iv: bytesToBase64(nonce),
            payload: bytesToBase64(ciphertext),
            mac: bytesToBase64(mac),
            gzip: isGzip,
            timestamp: Date.now()
        };
    };

    /**
     * Decrypt encrypted package with 6-digit PIN
     */
    const decryptPackage = async (pkg, pin) => {
        if (pkg.v === 2) {
            const derived = jsPbkdf2(pin, SALT, 5000, 64);
            const encKey = derived.slice(0, 32);
            const authKey = derived.slice(32, 64);

            const nonce = base64ToBytes(pkg.iv);
            const ciphertext = base64ToBytes(pkg.payload);
            const expectedMac = base64ToBytes(pkg.mac);

            // Verify HMAC-SHA256 authentication tag
            const computedMac = jsHmacSha256(authKey, ciphertext);
            let macMatches = (expectedMac.length === computedMac.length);
            for (let i = 0; i < computedMac.length; i++) {
                if (expectedMac[i] !== computedMac[i]) macMatches = false;
            }
            if (!macMatches) {
                throw new Error('Authentication failed: Invalid sync code or tampered data.');
            }

            const decrypted = jsChaCha20(encKey, nonce, 1, ciphertext);
            const decompressed = await decompressBytes(decrypted, pkg.gzip);
            return JSON.parse(decompressed);
        }

        // Backwards compatibility for v1 AES-GCM (if native subtle crypto is available)
        if (pkg.v === 1 && window.crypto && window.crypto.subtle) {
            const pinBytes = new TextEncoder().encode(pin);
            const baseKey = await crypto.subtle.importKey('raw', pinBytes, { name: 'PBKDF2' }, false, ['deriveKey']);
            const key = await crypto.subtle.deriveKey(
                { name: 'PBKDF2', salt: SALT, iterations: 100000, hash: 'SHA-256' },
                baseKey,
                { name: 'AES-GCM', length: 256 },
                false,
                ['decrypt']
            );
            const iv = base64ToBytes(pkg.iv);
            const ciphertext = base64ToBytes(pkg.payload);
            const decrypted = await crypto.subtle.decrypt({ name: 'AES-GCM', iv: iv }, key, ciphertext);
            const decompressed = await decompressBytes(new Uint8Array(decrypted), pkg.gzip);
            return JSON.parse(decompressed);
        }

        throw new Error('Unsupported package version or environment.');
    };

    /* ---------------- Database Merge Engine ---------------- */

    const mergeDatabases = (local, remote) => {
        if (!remote || typeof remote !== 'object') return local;
        if (!local || typeof local !== 'object') return remote;

        const merged = { ...local };

        const localTime = new Date(local.lastUpdated || 0).getTime();
        const remoteTime = new Date(remote.lastUpdated || remote.exportedAt || 0).getTime();

        if (remoteTime > localTime) {
            merged.profile = { ...local.profile, ...remote.profile };
            merged.settings = { ...local.settings, ...remote.settings };
        }

        // Merge clients by ID
        const clientMap = new Map();
        (local.clients || []).forEach(c => clientMap.set(c.id, c));
        (remote.clients || []).forEach(c => {
            if (!clientMap.has(c.id)) {
                clientMap.set(c.id, c);
            } else {
                const cur = clientMap.get(c.id);
                clientMap.set(c.id, { ...cur, ...c });
            }
        });
        merged.clients = Array.from(clientMap.values());

        // Merge invoices by ID & invoiceNumber
        const invoiceMap = new Map();
        (local.invoices || []).forEach(inv => invoiceMap.set(inv.id || inv.invoiceNumber, inv));
        (remote.invoices || []).forEach(inv => {
            const key = inv.id || inv.invoiceNumber;
            if (!invoiceMap.has(key)) {
                invoiceMap.set(key, inv);
            } else {
                const cur = invoiceMap.get(key);
                const curUpdated = new Date(cur.updatedAt || cur.date || 0).getTime();
                const remoteUpdated = new Date(inv.updatedAt || inv.date || 0).getTime();
                invoiceMap.set(key, remoteUpdated >= curUpdated ? inv : cur);
            }
        });
        merged.invoices = Array.from(invoiceMap.values());

        merged.lastUpdated = new Date(Math.max(localTime, remoteTime, Date.now())).toISOString();
        return merged;
    };

    /* ---------------- Network Transmission (ntfy.sh Relay) ---------------- */

    /**
     * Compute private channel name from PIN
     */
    const getTopicName = async (pin) => {
        const hash = await sha256Hex('JHA_TOPIC_' + pin);
        return 'jha-sync-' + hash.slice(0, 18);
    };

    /**
     * Send current database to relay with 5-minute expiry
     */
    const generateAndPublishCode = async () => {
        if (isSending) return;
        isSending = true;

        try {
            // Generate clean 6-digit numeric PIN (e.g. 582914)
            const pinNumber = Math.floor(100000 + Math.random() * 900000).toString();
            activeSyncCode = pinNumber;

            // Prepare database package
            const localData = InvoiceDB.getData();
            const encryptedBlob = await encryptPackage(localData, pinNumber);

            const topic = await getTopicName(pinNumber);
            const publishUrl = `${RELAY_BASE}/${topic}`;

            // Send to ephemeral relay (with 5-minute expiration)
            const res = await fetch(publishUrl, {
                method: 'POST',
                headers: {
                    'Title': 'JHA Sync Packet',
                    'Tags': 'key',
                    'Expires': '5m'
                },
                body: JSON.stringify(encryptedBlob)
            });

            if (!res.ok) {
                throw new Error(`Relay returned status ${res.status}`);
            }

            // Start 5-minute countdown timer
            startExpiryTimer(CODE_EXPIRY_SECONDS);
            renderCodeUI(pinNumber);

            if (typeof window.showToast === 'function') {
                window.showToast('Sync code generated! Valid for 5 minutes.', 'success');
            }
        } catch (err) {
            console.error('[SyncCode] Publish error:', err);
            alert('Could not generate sync code: ' + err.message + '\nPlease check your internet connection.');
        } finally {
            isSending = false;
        }
    };

    /**
     * Receive database using 6-digit code
     */
    const receiveAndSync = async (pin) => {
        const cleanedPin = (pin || '').replace(/\D/g, '').trim();
        if (cleanedPin.length !== 6) {
            alert('Please enter a valid 6-digit sync code.');
            return false;
        }

        if (isReceiving) return;
        isReceiving = true;
        updateReceiveButtonState(true);

        try {
            const topic = await getTopicName(cleanedPin);
            const pollUrl = `${RELAY_BASE}/${topic}/json?poll=1`;

            const res = await fetch(pollUrl);
            if (!res.ok) {
                throw new Error('Code not found or expired.');
            }

            const textData = await res.text();
            // ntfy.sh returns newline-delimited JSON messages; get the last valid one
            const lines = textData.trim().split('\n').filter(Boolean);
            if (lines.length === 0) {
                throw new Error('No sync data found for this code. Has the 5-minute timer expired?');
            }

            const lastMessage = JSON.parse(lines[lines.length - 1]);
            const encryptedPkg = JSON.parse(lastMessage.message);

            // Decrypt using PIN
            const remoteData = await decryptPackage(encryptedPkg, cleanedPin);
            const localData = InvoiceDB.getData();
            const merged = mergeDatabases(localData, remoteData);
            await InvoiceDB.save(merged);

            const msg = `Synced successfully! (${merged.invoices.length} invoices, ${merged.clients.length} clients)`;
            if (typeof window.showToast === 'function') {
                window.showToast(msg, 'success');
            } else {
                alert(msg);
            }

            // Clean reload to apply synced data
            setTimeout(() => location.reload(), 600);
            return true;
        } catch (err) {
            console.error('[SyncCode] Receive error:', err);
            alert(`Sync Failed: ${err.message}\nMake sure the code is correct and was generated within the last 5 minutes.`);
            return false;
        } finally {
            isReceiving = false;
            updateReceiveButtonState(false);
        }
    };

    /* ---------------- Timer & UI Rendering ---------------- */

    const startExpiryTimer = (durationSeconds) => {
        if (timerInterval) clearInterval(timerInterval);
        secondsRemaining = durationSeconds;

        updateTimerDisplay();

        timerInterval = setInterval(() => {
            secondsRemaining--;
            if (secondsRemaining <= 0) {
                clearInterval(timerInterval);
                timerInterval = null;
                secondsRemaining = 0;
                handleCodeExpired();
            }
            updateTimerDisplay();
        }, 1000);
    };

    const updateTimerDisplay = () => {
        const mins = Math.floor(secondsRemaining / 60);
        const secs = secondsRemaining % 60;
        const formatted = `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
        const pct = ((secondsRemaining / CODE_EXPIRY_SECONDS) * 100).toFixed(1);

        const timerEls = [
            document.getElementById('sidebar-sync-timer'),
            document.getElementById('settings-sync-timer')
        ];
        const barEls = [
            document.getElementById('sidebar-sync-progress'),
            document.getElementById('settings-sync-progress')
        ];

        timerEls.forEach(el => {
            if (el) el.textContent = formatted;
        });
        barEls.forEach(el => {
            if (el) el.style.width = `${pct}%`;
        });
    };

    const handleCodeExpired = () => {
        activeSyncCode = null;
        const displays = [
            document.getElementById('sidebar-code-digits'),
            document.getElementById('settings-code-digits')
        ];
        displays.forEach(el => {
            if (el) el.innerHTML = '<span class="code-expired-text">EXPIRED</span>';
        });

        const statusEls = [
            document.getElementById('sidebar-code-status'),
            document.getElementById('settings-code-status')
        ];
        statusEls.forEach(el => {
            if (el) el.textContent = '5-minute window expired. Generate a new code below.';
        });
    };

    const renderCodeUI = (pin) => {
        const formatted = `${pin.slice(0, 3)} ${pin.slice(3)}`;
        const displays = [
            document.getElementById('sidebar-code-digits'),
            document.getElementById('settings-code-digits')
        ];
        displays.forEach(el => {
            if (el) el.textContent = formatted;
        });

        const statusEls = [
            document.getElementById('sidebar-code-status'),
            document.getElementById('settings-code-status')
        ];
        statusEls.forEach(el => {
            if (el) el.textContent = 'Valid for 5 minutes. Enter this code on your other device.';
        });

        // Update stats
        const data = InvoiceDB.getData();
        const invCount = (data.invoices || []).length;
        const clientCount = (data.clients || []).length;
        const statsEls = [
            document.getElementById('sidebar-sync-stats'),
            document.getElementById('settings-sync-stats')
        ];
        statsEls.forEach(el => {
            if (el) el.textContent = `${invCount} Invoices • ${clientCount} Clients ready to sync`;
        });
    };

    const updateReceiveButtonState = (loading) => {
        const btns = [
            document.getElementById('sidebar-submit-code-btn'),
            document.getElementById('settings-submit-code-btn')
        ];
        btns.forEach(btn => {
            if (btn) {
                btn.disabled = loading;
                btn.innerHTML = loading
                    ? '<i class="ph ph-circle-notch qr-spin"></i> <span>Syncing...</span>'
                    : '<i class="ph ph-arrow-down-left"></i> <span>Connect &amp; Sync</span>';
            }
        });
    };

    const copyActiveCode = async () => {
        if (!activeSyncCode) {
            alert('Please generate a sync code first.');
            return;
        }
        try {
            await navigator.clipboard.writeText(activeSyncCode);
            if (typeof window.showToast === 'function') {
                window.showToast(`Code ${activeSyncCode} copied to clipboard!`, 'info');
            } else {
                alert(`Code ${activeSyncCode} copied!`);
            }
        } catch (e) {
            console.error('Clipboard copy error:', e);
        }
    };

    /* ---------------- Init UI Listeners ---------------- */

    const initUI = () => {
        // Generate buttons
        const genBtns = [
            document.getElementById('sidebar-gen-code-btn'),
            document.getElementById('settings-gen-code-btn')
        ];
        genBtns.forEach(btn => {
            if (btn) btn.addEventListener('click', generateAndPublishCode);
        });

        // Copy buttons
        const copyBtns = [
            document.getElementById('sidebar-copy-code-btn'),
            document.getElementById('settings-copy-code-btn')
        ];
        copyBtns.forEach(btn => {
            if (btn) btn.addEventListener('click', copyActiveCode);
        });

        // Receive / Submit buttons
        const submitSidebarBtn = document.getElementById('sidebar-submit-code-btn');
        const inputSidebar = document.getElementById('sidebar-input-code');
        if (submitSidebarBtn && inputSidebar) {
            submitSidebarBtn.addEventListener('click', () => {
                receiveAndSync(inputSidebar.value);
            });
            inputSidebar.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') receiveAndSync(inputSidebar.value);
            });
        }

        const submitSettingsBtn = document.getElementById('settings-submit-code-btn');
        const inputSettings = document.getElementById('settings-input-code');
        if (submitSettingsBtn && inputSettings) {
            submitSettingsBtn.addEventListener('click', () => {
                receiveAndSync(inputSettings.value);
            });
            inputSettings.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') receiveAndSync(inputSettings.value);
            });
        }

        // Auto-generate code when Settings tab opens if none is active
        const settingsTabBtn = document.getElementById('tab-btn-settings');
        if (settingsTabBtn) {
            settingsTabBtn.addEventListener('click', () => {
                if (!activeSyncCode || secondsRemaining <= 0) {
                    setTimeout(() => generateAndPublishCode(), 200);
                }
            });
        }
    };

    return {
        generateAndPublishCode,
        receiveAndSync,
        copyActiveCode,
        initUI
    };
})();

// Auto-init on DOM load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => SyncCode.initUI());
} else {
    SyncCode.initUI();
}
