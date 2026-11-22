/**
 * JHA Invoice Maker - Rust PDF engine loader (pdf-engine.js)
 *
 * Loads pdf-engine.wasm: the desktop app's Rust invoice layout, vector PDF
 * writer and AES-256 edit protection compiled to WebAssembly (source in
 * web-engine/). Produces the same single-page PDF as the desktop app:
 * selectable text, embedded Lexend fonts, vector logo.
 *
 *   const bytes = await JhaPdfEngine.exportPdf({ profile, invoice, password });
 */
(function () {
    'use strict';

    const WASM_URL = 'pdf-engine.wasm?v=1.0.0';
    let enginePromise = null;

    function load() {
        if (enginePromise) return enginePromise;
        enginePromise = (async () => {
            let memory = null;
            const imports = {
                env: {
                    // Entropy for AES keys and document IDs.
                    jha_random_fill(ptr, len) {
                        const view = new Uint8Array(memory.buffer, ptr, len);
                        for (let off = 0; off < len; off += 65536) {
                            crypto.getRandomValues(view.subarray(off, Math.min(len, off + 65536)));
                        }
                    }
                }
            };

            let result = null;
            if (WebAssembly.instantiateStreaming) {
                try {
                    result = await WebAssembly.instantiateStreaming(fetch(WASM_URL), imports);
                } catch (e) {
                    result = null; // e.g. wrong MIME type: fall back to ArrayBuffer
                }
            }
            if (!result) {
                const res = await fetch(WASM_URL);
                if (!res.ok) throw new Error(`Could not load the PDF engine (HTTP ${res.status})`);
                result = await WebAssembly.instantiate(await res.arrayBuffer(), imports);
            }
            const exports = result.instance.exports;
            memory = exports.memory;
            return exports;
        })();
        enginePromise.catch(() => { enginePromise = null; });
        return enginePromise;
    }

    async function exportPdf(request) {
        const ex = await load();
        const input = new TextEncoder().encode(JSON.stringify(request));
        let status;
        try {
            const ptr = ex.jha_alloc(input.length);
            new Uint8Array(ex.memory.buffer, ptr, input.length).set(input);
            try {
                status = ex.jha_export_pdf(ptr, input.length);
            } finally {
                ex.jha_free(ptr, input.length);
            }
        } catch (err) {
            // A trap leaves the instance unusable; start fresh next time.
            enginePromise = null;
            throw new Error(`PDF engine crashed: ${err.message || err}`);
        }
        const out = new Uint8Array(ex.memory.buffer, ex.jha_result_ptr(), ex.jha_result_len()).slice();
        ex.jha_result_clear();
        if (status !== 0) throw new Error(new TextDecoder().decode(out));
        return out;
    }

    window.JhaPdfEngine = {
        load,
        exportPdf,
        /** Start downloading the engine in the background (no error if it fails). */
        preload() { load().catch(() => {}); }
    };
})();
