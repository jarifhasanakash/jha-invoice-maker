/**
 * JHA Invoice Maker - JSON Database Engine (db.js)
 * High-reliability browser-native JSON Database powered by IndexedDB with LocalStorage mirroring.
 * Works seamlessly across all desktop & mobile browsers (Windows, Mac, iOS, Android, Linux).
 */

const InvoiceDB = (() => {
    const DB_NAME = 'JHA_INVOICE_MAKER_DB';
    const DB_VERSION = 1;
    const STORE_NAME = 'app_data';
    const DATA_KEY = 'main_json_database';
    const MIRROR_KEY = 'jha_invoice_db_mirror';

    // Default template schema for new installations
    const DEFAULT_SCHEMA = {
        version: '2.0',
        lastUpdated: new Date().toISOString(),
        profile: {
            firstName: 'John',
            lastName: 'Doe',
            email: 'john.doe@example.com',
            mobile: '+1 (555) 234-5678',
            address: '742 Evergreen Terrace, Springfield, OR',
            companyLogo: null,
            platforms: [
                { id: 'platform1', name: 'PayPal', email: 'john.doe@example.com' },
                { id: 'platform2', name: 'Wise', email: 'john.doe@example.com' }
            ],
            bank: {
                bankName: 'Chase Bank',
                accountNumber: '9876543210',
                branchName: 'Downtown Branch',
                branchCode: '102',
                swiftCode: 'CHASUS33XXX',
                routingNo: '021000021'
            }
        },
        clients: [
            {
                id: 'client_sample_1',
                name: 'Acme Corporation',
                location: 'San Francisco, CA, USA',
                email: 'billing@acme.example.com',
                phone: '+1 415-555-0199',
                notes: 'Payment terms: 14 days'
            }
        ],
        invoices: [],
        settings: {
            lastInvoiceNumber: '0001',
            defaultDueDays: 14,
            currencySymbol: '$',
            defaultNote: 'Please complete your payment within 14 days, your cooperation is greatly appreciated.',
            autoBackupReminder: true
        }
    };

    let dbInstance = null;
    let cachedData = null;
    let changeListeners = [];

    // Helper: open IndexedDB connection
    const openDB = () => {
        return new Promise((resolve, reject) => {
            if (dbInstance) {
                return resolve(dbInstance);
            }
            if (!window.indexedDB) {
                console.warn('[InvoiceDB] IndexedDB not available in this environment. Falling back to LocalStorage mirror.');
                return resolve(null);
            }

            const request = indexedDB.open(DB_NAME, DB_VERSION);

            request.onupgradeneeded = (event) => {
                const db = event.target.result;
                if (!db.objectStoreNames.contains(STORE_NAME)) {
                    db.createObjectStore(STORE_NAME);
                }
            };

            request.onsuccess = (event) => {
                dbInstance = event.target.result;
                resolve(dbInstance);
            };

            request.onerror = (event) => {
                console.error('[InvoiceDB] Failed to open IndexedDB:', event.target.error);
                resolve(null); // gracefully fall back to localStorage
            };
        });
    };

    // Helper: migrate data from legacy localStorage keys if present
    const migrateLegacyData = (schema) => {
        let modified = false;
        try {
            const legacyBank = localStorage.getItem('bankDetails');
            const legacyLogo = localStorage.getItem('companyLogo');
            const legacyClients = localStorage.getItem('clientList');
            const legacyHistory = localStorage.getItem('invoiceHistory');
            const legacyInvNum = localStorage.getItem('lastInvoiceNumber');

            if (legacyBank) {
                const parsedBank = JSON.parse(legacyBank);
                schema.profile.firstName = parsedBank.firstName || schema.profile.firstName;
                schema.profile.lastName = parsedBank.lastName || schema.profile.lastName;
                schema.profile.email = parsedBank.email || schema.profile.email;
                schema.profile.mobile = parsedBank.mobile || schema.profile.mobile;
                schema.profile.address = parsedBank.address || schema.profile.address;

                schema.profile.platforms = [
                    { id: 'platform1', name: (parsedBank.platform1Name === 'Payoneer' ? 'PayPal' : (parsedBank.platform1Name || 'PayPal')), email: parsedBank.platform1Email || schema.profile.email },
                    { id: 'platform2', name: parsedBank.platform2Name || 'Wise', email: parsedBank.platform2Email || schema.profile.email }
                ];

                schema.profile.bank = {
                    bankName: parsedBank.bankName || schema.profile.bank.bankName,
                    accountNumber: parsedBank.accountNumber || schema.profile.bank.accountNumber,
                    branchName: parsedBank.branchName || schema.profile.bank.branchName,
                    branchCode: parsedBank.branchCode || schema.profile.bank.branchCode,
                    swiftCode: parsedBank.swiftCode || schema.profile.bank.swiftCode,
                    routingNo: parsedBank.routingNo || schema.profile.bank.routingNo
                };
                modified = true;
            }

            if (legacyLogo) {
                schema.profile.companyLogo = legacyLogo;
                modified = true;
            }

            if (legacyClients) {
                const parsedClients = JSON.parse(legacyClients);
                if (Array.isArray(parsedClients) && parsedClients.length > 0) {
                    schema.clients = parsedClients.map((c, idx) => ({
                        id: c.id || `client_${Date.now()}_${idx}`,
                        name: c.name || 'Unnamed Client',
                        location: c.location || '',
                        email: c.email || '',
                        phone: c.phone || '',
                        notes: c.notes || ''
                    }));
                    modified = true;
                }
            }

            if (legacyHistory) {
                const parsedHistory = JSON.parse(legacyHistory);
                if (Array.isArray(parsedHistory) && parsedHistory.length > 0) {
                    schema.invoices = parsedHistory.map((item, idx) => ({
                        id: item.id || `inv_${Date.now()}_${idx}`,
                        invoiceNumber: item.number || String(idx + 1).padStart(4, '0'),
                        date: item.date || new Date().toISOString().split('T')[0],
                        dueDate: item.dueDate || item.date || new Date().toISOString().split('T')[0],
                        status: item.status || 'Paid',
                        client: {
                            name: item.client || 'Client',
                            location: item.location || '',
                            email: item.email || ''
                        },
                        items: item.items || [{ description: 'Invoice Item', sub: '', price: item.amount || 0 }],
                        toggleAdvance: !!item.toggleAdvance,
                        advanceAmount: item.advanceAmount || 0,
                        toggleBalance: !!item.toggleBalance,
                        total: item.amount || 0,
                        balance: item.balance !== undefined ? item.balance : (item.amount || 0),
                        note: item.note || schema.settings.defaultNote,
                        createdAt: item.timestamp || new Date().toISOString(),
                        updatedAt: item.timestamp || new Date().toISOString()
                    }));
                    modified = true;
                }
            }

            if (legacyInvNum) {
                schema.settings.lastInvoiceNumber = legacyInvNum;
                modified = true;
            }
        } catch (err) {
            console.error('[InvoiceDB] Legacy migration warning:', err);
        }
        return modified;
    };

    const CURRENT_SCHEMA_VERSION = '2.1';

    // Semantic version comparator helper (returns -1 if v1 < v2, 0 if equal, 1 if v1 > v2)
    const compareVersions = (v1, v2) => {
        if (!v1) return -1;
        if (!v2) return 1;
        const p1 = String(v1).split('.').map(n => parseInt(n, 10) || 0);
        const p2 = String(v2).split('.').map(n => parseInt(n, 10) || 0);
        const len = Math.max(p1.length, p2.length);
        for (let i = 0; i < len; i++) {
            const num1 = p1[i] || 0;
            const num2 = p2[i] || 0;
            if (num1 < num2) return -1;
            if (num1 > num2) return 1;
        }
        return 0;
    };

    // Deep merge helper that recursively preserves incoming custom/legacy properties
    // while supplying defaults for any newly introduced keys in the active schema.
    const deepMerge = (target, source) => {
        if (!source || typeof source !== 'object') return target;
        const result = Array.isArray(target) ? [...target] : Object.assign({}, target);

        Object.keys(source).forEach(key => {
            const srcVal = source[key];
            const tgtVal = target ? target[key] : undefined;

            if (srcVal === null || srcVal === undefined) {
                result[key] = srcVal;
            } else if (Array.isArray(srcVal)) {
                // If it's an array, preserve source array items completely
                result[key] = srcVal;
            } else if (typeof srcVal === 'object' && typeof tgtVal === 'object') {
                result[key] = deepMerge(tgtVal, srcVal);
            } else {
                result[key] = srcVal;
            }
        });
        return result;
    };

    // Sequential schema migrations pipeline
    // To add new features in the future, simply add a new step here!
    const MIGRATIONS = [
        {
            toVersion: '2.0',
            upgrade(data) {
                if (!data.profile) data.profile = {};
                if (!data.settings) data.settings = {};
                if (!Array.isArray(data.clients)) data.clients = [];
                if (!Array.isArray(data.invoices)) data.invoices = [];
                return data;
            }
        },
        {
            toVersion: '2.1',
            upgrade(data) {
                // 1. Future-proof invoice line items and calculation fields
                if (Array.isArray(data.invoices)) {
                    data.invoices = data.invoices.map((inv, idx) => {
                        const items = Array.isArray(inv.items) && inv.items.length > 0
                            ? inv.items.map(it => ({
                                description: it.description || '',
                                sub: it.sub || '',
                                price: typeof it.price === 'number' ? it.price : (parseFloat(it.price) || 0)
                            }))
                            : [{ description: 'Website Development', sub: '', price: inv.total || 0 }];

                        const total = inv.total !== undefined ? inv.total : items.reduce((sum, i) => sum + (i.price || 0), 0);
                        const advance = inv.advanceAmount || 0;
                        const balance = inv.balance !== undefined ? inv.balance : (total - advance);

                        return {
                            id: inv.id || `inv_${Date.now()}_${idx}`,
                            invoiceNumber: inv.invoiceNumber || String(idx + 1).padStart(4, '0'),
                            date: inv.date || new Date().toISOString().split('T')[0],
                            dueDate: inv.dueDate || inv.date || new Date().toISOString().split('T')[0],
                            status: inv.status || 'Pending',
                            from: inv.from || { name: '', location: '', email: '' },
                            client: inv.client || inv.to || { name: 'Client Name', location: '', email: '' },
                            items: items,
                            toggleAdvance: !!inv.toggleAdvance,
                            advanceAmount: advance,
                            toggleBalance: !!inv.toggleBalance,
                            total: total,
                            balance: balance,
                            note: inv.note || 'Please complete your payment within 14 days, your cooperation is greatly appreciated.',
                            createdAt: inv.createdAt || new Date().toISOString(),
                            updatedAt: inv.updatedAt || new Date().toISOString(),
                            customFields: inv.customFields || {}
                        };
                    });
                }

                // 2. Future-proof client records
                if (Array.isArray(data.clients)) {
                    data.clients = data.clients.map((c, idx) => ({
                        id: c.id || `client_${Date.now()}_${idx}`,
                        name: c.name || 'Client Name',
                        location: c.location || '',
                        email: c.email || '',
                        phone: c.phone || '',
                        notes: c.notes || '',
                        createdAt: c.createdAt || new Date().toISOString(),
                        updatedAt: c.updatedAt || new Date().toISOString(),
                        customFields: c.customFields || {}
                    }));
                }

                // 3. Ensure customFields on profile and settings
                if (data.profile && !data.profile.customFields) {
                    data.profile.customFields = {};
                }
                if (data.settings && !data.settings.customFields) {
                    data.settings.customFields = {};
                }
                return data;
            }
        }
    ];

    // Auto-migration runner: applies any applicable migrations sequentially
    const runMigrations = (data) => {
        if (!data) return data;
        let version = data.schemaVersion || data.version || '1.0';

        MIGRATIONS.forEach(m => {
            if (compareVersions(version, m.toVersion) < 0) {
                try {
                    data = m.upgrade(data);
                    version = m.toVersion;
                } catch (e) {
                    console.warn(`[InvoiceDB] Migration to ${m.toVersion} warning:`, e);
                }
            }
        });

        // Deep merge with current DEFAULT_SCHEMA to ensure newly introduced default properties exist
        const merged = deepMerge(DEFAULT_SCHEMA, data);
        merged.schemaVersion = CURRENT_SCHEMA_VERSION;
        merged.version = CURRENT_SCHEMA_VERSION;
        merged.dataVersion = (data.dataVersion || 1) + 1;
        merged.lastUpdated = new Date().toISOString();
        return merged;
    };

    // Public API
    return {
        /**
         * Initialize DB and return the active JSON dataset
         */
        async init() {
            const db = await openDB();
            let loadedData = null;

            // 1. Try reading from IndexedDB
            if (db) {
                try {
                    loadedData = await new Promise((resolve, reject) => {
                        const tx = db.transaction([STORE_NAME], 'readonly');
                        const store = tx.objectStore(STORE_NAME);
                        const req = store.get(DATA_KEY);
                        req.onsuccess = () => resolve(req.result);
                        req.onerror = () => reject(req.error);
                    });
                } catch (err) {
                    console.warn('[InvoiceDB] Error reading from IndexedDB:', err);
                }
            }

            // 2. If not found in IndexedDB, try localStorage mirror
            if (!loadedData) {
                try {
                    const mirror = localStorage.getItem(MIRROR_KEY);
                    if (mirror) {
                        loadedData = JSON.parse(mirror);
                    }
                } catch (err) {
                    console.warn('[InvoiceDB] Error reading localStorage mirror:', err);
                }
            }

            // 3. If still not found, check legacy migration or initialize default
            if (!loadedData) {
                const fresh = JSON.parse(JSON.stringify(DEFAULT_SCHEMA));
                migrateLegacyData(fresh);
                loadedData = runMigrations(fresh);
                await this.save(loadedData);
            } else {
                // Ensure schema completeness & run migrations
                const migrated = runMigrations(loadedData);
                loadedData = migrated;
                await this.save(loadedData);
            }

            cachedData = loadedData;
            return cachedData;
        },

        /**
         * Get cached database object (synchronous)
         */
        getData() {
            if (!cachedData) {
                const mirror = localStorage.getItem(MIRROR_KEY);
                if (mirror) {
                    try { cachedData = JSON.parse(mirror); } catch (e) { }
                }
                if (!cachedData) cachedData = JSON.parse(JSON.stringify(DEFAULT_SCHEMA));
            }
            return cachedData;
        },

        /**
         * Persist data to IndexedDB and LocalStorage mirror
         */
        async save(data) {
            data.lastUpdated = new Date().toISOString();
            cachedData = data;

            // Save to localStorage mirror synchronously
            try {
                localStorage.setItem(MIRROR_KEY, JSON.stringify(data));
                // Maintain legacy sync for backward compatibility
                if (data.settings && data.settings.lastInvoiceNumber) {
                    localStorage.setItem('lastInvoiceNumber', data.settings.lastInvoiceNumber);
                }
            } catch (err) {
                console.warn('[InvoiceDB] Failed to write to localStorage mirror:', err);
            }

            // Save to IndexedDB asynchronously
            const db = await openDB();
            if (db) {
                try {
                    await new Promise((resolve, reject) => {
                        const tx = db.transaction([STORE_NAME], 'readwrite');
                        const store = tx.objectStore(STORE_NAME);
                        const req = store.put(data, DATA_KEY);
                        req.onsuccess = () => resolve();
                        req.onerror = () => reject(req.error);
                    });
                } catch (err) {
                    console.error('[InvoiceDB] Failed to write to IndexedDB:', err);
                }
            }

            // Trigger change listeners
            changeListeners.forEach(listener => {
                try { listener(cachedData); } catch (e) { console.error(e); }
            });

            return cachedData;
        },

        /**
         * Subscribe to database updates
         */
        onChange(fn) {
            if (typeof fn === 'function') {
                changeListeners.push(fn);
            }
        },

        /* ---------------- Clients Management ---------------- */

        getClients() {
            const data = this.getData();
            return data.clients || [];
        },

        getClient(id) {
            const clients = this.getClients();
            return clients.find(c => c.id === id || c.name === id) || null;
        },

        async saveClient(client) {
            const data = this.getData();
            if (!Array.isArray(data.clients)) data.clients = [];

            if (!client.id) {
                client.id = 'client_' + Date.now();
            }

            const existingIndex = data.clients.findIndex(c => c.id === client.id || c.name.toLowerCase() === client.name.toLowerCase());
            if (existingIndex > -1) {
                data.clients[existingIndex] = Object.assign({}, data.clients[existingIndex], client, {
                    updatedAt: new Date().toISOString()
                });
            } else {
                client.createdAt = new Date().toISOString();
                client.updatedAt = new Date().toISOString();
                data.clients.push(client);
            }

            await this.save(data);
            return client;
        },

        async deleteClient(clientId) {
            const data = this.getData();
            data.clients = (data.clients || []).filter(c => c.id !== clientId);
            await this.save(data);
            return true;
        },

        /* ---------------- Invoices Management ---------------- */

        getInvoices() {
            const data = this.getData();
            return data.invoices || [];
        },

        getInvoice(idOrNumber) {
            const invoices = this.getInvoices();
            return invoices.find(inv => inv.id === idOrNumber || inv.invoiceNumber === idOrNumber) || null;
        },

        async saveInvoice(invoice) {
            const data = this.getData();
            if (!Array.isArray(data.invoices)) data.invoices = [];

            if (!invoice.id) {
                invoice.id = 'inv_' + Date.now();
                invoice.createdAt = new Date().toISOString();
            }
            invoice.updatedAt = new Date().toISOString();

            const existingIndex = data.invoices.findIndex(inv => inv.id === invoice.id || inv.invoiceNumber === invoice.invoiceNumber);
            if (existingIndex > -1) {
                data.invoices[existingIndex] = Object.assign({}, data.invoices[existingIndex], invoice);
            } else {
                data.invoices.push(invoice);
            }

            // Update last invoice number setting if this invoice has a higher number
            const num = parseInt(invoice.invoiceNumber, 10);
            const currentLast = parseInt(data.settings.lastInvoiceNumber || '0', 10);
            if (!isNaN(num) && num > currentLast) {
                data.settings.lastInvoiceNumber = String(num).padStart(4, '0');
            }

            await this.save(data);
            return invoice;
        },

        async deleteInvoice(idOrNumber) {
            const data = this.getData();
            data.invoices = (data.invoices || []).filter(inv => inv.id !== idOrNumber && inv.invoiceNumber !== idOrNumber);
            await this.save(data);
            return true;
        },

        async updateInvoiceStatus(idOrNumber, newStatus) {
            const data = this.getData();
            const inv = (data.invoices || []).find(i => i.id === idOrNumber || i.invoiceNumber === idOrNumber);
            if (inv) {
                inv.status = newStatus;
                inv.updatedAt = new Date().toISOString();
                await this.save(data);
                return inv;
            }
            return null;
        },

        /* ---------------- Profile & Settings ---------------- */

        getProfile() {
            return this.getData().profile || DEFAULT_SCHEMA.profile;
        },

        async saveProfile(profile) {
            const data = this.getData();
            data.profile = Object.assign({}, data.profile, profile);
            await this.save(data);
            return data.profile;
        },

        getSettings() {
            return this.getData().settings || DEFAULT_SCHEMA.settings;
        },

        async saveSettings(settings) {
            const data = this.getData();
            data.settings = Object.assign({}, data.settings, settings);
            await this.save(data);
            return data.settings;
        },

        /* ---------------- Backup, Export & Restore ---------------- */

        /**
         * Export entire database as formatted JSON file
         */
        exportDatabaseJSON() {
            const data = this.getData();
            const jsonStr = JSON.stringify(data, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const dateStr = new Date().toISOString().split('T')[0];
            const filename = `jha_invoice_database_${dateStr}.json`;

            const link = document.createElement('a');
            link.href = URL.createObjectURL(blob);
            link.download = filename;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(link.href);
            return filename;
        },

        /**
         * Import & restore database from JSON file content
         */
        async importDatabaseJSON(fileContent) {
            let parsed;
            try {
                parsed = JSON.parse(fileContent);
            } catch (e) {
                throw new Error('Invalid JSON format: File is corrupted or not a valid JSON document.');
            }

            // Check if it is a structured schema or legacy format
            let restoredData = null;

            if (parsed.profile || parsed.clients || parsed.invoices) {
                // v2.0+ format: run through migration pipeline to auto-upgrade to latest schema
                restoredData = runMigrations(parsed);
            } else if (parsed.bankDetails || parsed.clientList || parsed.invoiceHistory) {
                // Legacy v1 format
                const legacy = JSON.parse(JSON.stringify(DEFAULT_SCHEMA));
                if (parsed.bankDetails) {
                    const b = typeof parsed.bankDetails === 'string' ? JSON.parse(parsed.bankDetails) : parsed.bankDetails;
                    legacy.profile.firstName = b.firstName || legacy.profile.firstName;
                    legacy.profile.lastName = b.lastName || legacy.profile.lastName;
                    legacy.profile.email = b.email || legacy.profile.email;
                    legacy.profile.mobile = b.mobile || legacy.profile.mobile;
                    legacy.profile.address = b.address || legacy.profile.address;
                    legacy.profile.bank = {
                        bankName: b.bankName || '',
                        accountNumber: b.accountNumber || '',
                        branchName: b.branchName || '',
                        branchCode: b.branchCode || '',
                        swiftCode: b.swiftCode || '',
                        routingNo: b.routingNo || ''
                    };
                }
                if (parsed.companyLogo) legacy.profile.companyLogo = parsed.companyLogo;
                if (parsed.clientList) {
                    const cList = typeof parsed.clientList === 'string' ? JSON.parse(parsed.clientList) : parsed.clientList;
                    legacy.clients = (cList || []).map((c, i) => ({
                        id: 'client_' + i,
                        name: c.name || '',
                        location: c.location || '',
                        email: c.email || '',
                        phone: '',
                        notes: ''
                    }));
                }
                if (parsed.invoiceHistory) {
                    const hList = typeof parsed.invoiceHistory === 'string' ? JSON.parse(parsed.invoiceHistory) : parsed.invoiceHistory;
                    legacy.invoices = (hList || []).map((h, i) => ({
                        id: 'inv_' + i,
                        invoiceNumber: h.number || String(i + 1).padStart(4, '0'),
                        date: h.date || new Date().toISOString().split('T')[0],
                        dueDate: h.date || new Date().toISOString().split('T')[0],
                        status: 'Paid',
                        client: { name: h.client || '', location: '', email: '' },
                        items: [{ description: 'Website Development', sub: '', price: h.amount || 0 }],
                        toggleAdvance: false,
                        advanceAmount: 0,
                        toggleBalance: false,
                        total: h.amount || 0,
                        balance: h.amount || 0,
                        note: legacy.settings.defaultNote,
                        createdAt: h.timestamp || new Date().toISOString(),
                        updatedAt: h.timestamp || new Date().toISOString()
                    }));
                }
                if (parsed.lastInvoiceNumber) {
                    legacy.settings.lastInvoiceNumber = parsed.lastInvoiceNumber;
                }
                restoredData = runMigrations(legacy);
            } else {
                throw new Error('Unrecognized database format: Does not contain profile, clients, or invoices data.');
            }

            await this.save(restoredData);
            return {
                clientsCount: restoredData.clients.length,
                invoicesCount: restoredData.invoices.length,
                profileName: `${restoredData.profile.firstName} ${restoredData.profile.lastName}`.trim()
            };
        },

        /**
         * Export invoices list as CSV
         */
        exportInvoicesCSV(invoicesList) {
            const list = invoicesList || this.getInvoices();
            if (!list || list.length === 0) {
                throw new Error('No invoices to export.');
            }

            let csv = 'Invoice Number,Date,Due Date,Status,Client Name,Client Email,Amount,Advance,Balance\n';
            list.forEach(inv => {
                const invNo = `"${(inv.invoiceNumber || '').replace(/"/g, '""')}"`;
                const date = `"${inv.date || ''}"`;
                const dueDate = `"${inv.dueDate || ''}"`;
                const status = `"${inv.status || 'Pending'}"`;
                const clientName = `"${(inv.client?.name || inv.to?.name || inv.clientName || '').replace(/"/g, '""')}"`;
                const clientEmail = `"${(inv.client?.email || inv.to?.email || '').replace(/"/g, '""')}"`;
                const total = inv.total || 0;
                const advance = inv.advanceAmount || 0;
                const balance = inv.balance !== undefined ? inv.balance : (total - advance);

                csv += `${invNo},${date},${dueDate},${status},${clientName},${clientEmail},${total},${advance},${balance}\n`;
            });

            const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
            const link = document.createElement('a');
            link.href = URL.createObjectURL(blob);
            link.download = `invoices_export_${new Date().toISOString().split('T')[0]}.csv`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(link.href);
        },

        /**
         * Export invoices list as JSON
         */
        exportInvoicesJSON(invoicesList) {
            const list = invoicesList || this.getInvoices();
            const blob = new Blob([JSON.stringify(list, null, 2)], { type: 'application/json' });
            const link = document.createElement('a');
            link.href = URL.createObjectURL(blob);
            link.download = `invoices_export_${new Date().toISOString().split('T')[0]}.json`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(link.href);
        },

        /**
         * Reset database to default
         */
        async resetDatabase() {
            const fresh = JSON.parse(JSON.stringify(DEFAULT_SCHEMA));
            await this.save(fresh);
            // Clear legacy keys too
            try {
                localStorage.removeItem('companyLogo');
                localStorage.removeItem('bankDetails');
                localStorage.removeItem('clientList');
                localStorage.removeItem('invoiceHistory');
                localStorage.removeItem('lastInvoiceNumber');
            } catch (e) { }
            return fresh;
        }
    };
})();

if (typeof module !== 'undefined' && module.exports) {
    module.exports = InvoiceDB;
}

