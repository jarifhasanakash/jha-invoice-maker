/**
 * JHA Invoice Maker - Frontend Logic (script.js)
 * Coordinates the UI, invoice calculations, live preview, print engine,
 * and seamlessly communicates with the browser-native JSON Database (db.js).
 */

document.addEventListener('DOMContentLoaded', async () => {
    // 1. Initialize JSON Database Engine
    await InvoiceDB.init();

    // 2. State Variables
    let currentEditingInvoiceId = null;
    let currentInvoiceStatus = 'Pending';
    let currentClient = {
        id: null,
        name: 'Client Name',
        location: 'City/State, Country',
        email: 'client@example.com'
    };
    let items = [
        { description: 'Website Development', sub: 'https://websitename.com', price: 200 }
    ];
    let userModifiedItems = false;
    let currentZoom = 1.0;

    // 3. Cache DOM Elements
    const inputs = {
        invoiceNumber: document.getElementById('invoiceNumber'),
        invoiceDate: document.getElementById('invoiceDate'),
        clientSelect: document.getElementById('client-select'),
        toggleAdvance: document.getElementById('toggleAdvance'),
        advanceAmount: document.getElementById('advanceAmount'),
        toggleBalance: document.getElementById('toggleBalance')
    };

    // Date handling will initialize date value formatted below

    const previews = {
        number: document.getElementById('preview-number'),
        date: document.getElementById('preview-date'),
        clientNameHeader: document.getElementById('preview-client-name-header'),
        fromName: document.getElementById('preview-from-name'),
        fromLocation: document.getElementById('preview-from-location'),
        fromEmail: document.getElementById('preview-from-email'),
        toName: document.getElementById('preview-to-name'),
        toLocation: document.getElementById('preview-to-location'),
        toEmail: document.getElementById('preview-to-email'),
        itemsList: document.getElementById('preview-items-list'),
        total: document.getElementById('preview-total'),
        advance: document.getElementById('preview-advance'),
        balance: document.getElementById('preview-balance'),
        noteText: document.getElementById('preview-note-text'),
        rowAdvance: document.getElementById('row-advance'),
        rowBalance: document.getElementById('row-balance'),
        signatureName: document.getElementById('preview-signature-name'),
        mobileContainer: document.getElementById('preview-mobile-container'),
        mobileDivider: document.getElementById('preview-mobile-divider'),
        mobile: document.getElementById('preview-mobile'),
        statusPill: document.getElementById('preview-status-pill'),
        toolbarDocTitle: document.getElementById('toolbar-doc-title'),
        breakdownContainer: document.querySelector('.invoice-payment-breakdown')
    };

    const previewWrapper = document.querySelector('.preview-wrapper');
    const invoicePreview = document.getElementById('invoice-preview');
    const itemsContainer = document.getElementById('items-container');
    const itemsCountBadge = document.getElementById('items-count-badge');
    const addItemBtn = document.getElementById('add-item-btn');
    const advanceInputGroup = document.getElementById('advanceInputGroup');

    // Action buttons in top preview bar
    const saveInvoiceBtn = document.getElementById('save-invoice-btn');
    const printBtn = document.getElementById('print-btn');
    const newInvoiceBtn = document.getElementById('new-invoice-btn');
    const quickPrintBtn = document.getElementById('quick-print-btn');
    const quickExportJsonBtn = document.getElementById('quick-export-json-btn');

    // Editing banner in sidebar & preview toolbar chip
    const editingBanner = document.getElementById('editing-banner');
    const editingInvoiceNo = document.getElementById('editing-invoice-no');
    const cancelEditBtn = document.getElementById('cancel-edit-btn');
    const previewEditingChip = document.getElementById('preview-editing-chip');
    const cancelEditChipBtn = document.getElementById('cancel-edit-chip-btn');

    // Tab Navigation Elements (Tabs instead of popups)
    const tabBtnNew = document.getElementById('tab-btn-new');
    const tabBtnHistory = document.getElementById('tab-btn-history');
    const tabBtnClients = document.getElementById('tab-btn-clients');
    const tabBtnSettings = document.getElementById('tab-btn-settings');

    const tabPanes = {
        'tab-invoice': document.getElementById('view-invoice'),
        'tab-history': document.getElementById('view-history'),
        'tab-clients': document.getElementById('view-clients'),
        'tab-settings': document.getElementById('view-settings')
    };

    let activeTabKey = 'tab-invoice';
    let tabTransitionTimeout = null;
    const dbBadge = document.getElementById('db-badge');

    // Zoom buttons
    const zoomInBtn = document.getElementById('zoom-in-btn');
    const zoomOutBtn = document.getElementById('zoom-out-btn');
    const zoomFitBtn = document.getElementById('zoom-fit-btn');
    const zoomResetBtn = document.getElementById('zoom-reset-btn');
    const zoomLevelEl = document.getElementById('zoom-level');

    // Client modal elements
    const clientSearchInput = document.getElementById('client-search-input');
    const toggleAddClientBtn = document.getElementById('toggle-add-client-btn');
    const clientFormContainer = document.getElementById('client-form-container');
    const clientFormTitle = document.getElementById('client-form-title');
    const clientFormId = document.getElementById('client-form-id');
    const clientFormName = document.getElementById('client-form-name');
    const clientFormEmail = document.getElementById('client-form-email');
    const clientFormLocation = document.getElementById('client-form-location');
    const clientFormPhone = document.getElementById('client-form-phone');
    const clientFormCurrency = document.getElementById('client-form-currency');
    const clientFormNotes = document.getElementById('client-form-notes');
    const saveClientFormBtn = document.getElementById('save-client-form-btn');
    const cancelClientFormBtn = document.getElementById('cancel-client-form-btn');
    const clientsListContainer = document.getElementById('clients-list-container');
    const modalClientsCount = document.getElementById('modal-clients-count');

    // Settings elements
    const logoInput = document.getElementById('logoInput');
    const removeLogoBtn = document.getElementById('remove-logo');
    const logoPreviewBox = document.getElementById('logo-preview-box');
    const invoiceLogoContainer = document.querySelector('.invoice-logo');
    const exportDataBtn = document.getElementById('export-data-btn');
    const importInput = document.getElementById('importInput');
    const resetDataBtn = document.getElementById('reset-data-btn');

    const settingInputs = {
        firstName: document.getElementById('setting-firstName'),
        lastName: document.getElementById('setting-lastName'),
        email: document.getElementById('setting-email'),
        mobile: document.getElementById('setting-mobile'),
        address: document.getElementById('setting-address'),
        platform1Name: document.getElementById('setting-platform1Name'),
        platform1Email: document.getElementById('setting-platform1Email'),
        platform2Name: document.getElementById('setting-platform2Name'),
        platform2Email: document.getElementById('setting-platform2Email'),
        bankName: document.getElementById('setting-bankName'),
        accountNumber: document.getElementById('setting-accountNumber'),
        branchName: document.getElementById('setting-branchName'),
        branchCode: document.getElementById('setting-branchCode'),
        swiftCode: document.getElementById('setting-swiftCode'),
        routingNo: document.getElementById('setting-routingNo'),
        defaultNote: document.getElementById('setting-defaultNote'),
        currency: document.getElementById('setting-currency')
    };

    // Earnings & History elements
    const historySearchInput = document.getElementById('history-search-input');
    const historyStatusFilter = document.getElementById('history-status-filter');
    const historyFromInput = document.getElementById('history-from-date');
    const historyToInput = document.getElementById('history-to-date');
    const historyTableBody = document.getElementById('history-table-body');
    const exportCsvBtn = document.getElementById('export-csv-btn');
    const exportInvoicesJsonBtn = document.getElementById('export-invoices-json-btn');
    const statTotalInvoiced = document.getElementById('stat-total-invoiced');
    const statTotalPaid = document.getElementById('stat-total-paid');
    const statTotalPending = document.getElementById('stat-total-pending');
    const statInvoicesCount = document.getElementById('stat-invoices-count');

    // Toast Container
    const toastContainer = document.getElementById('toast-container');

    /* ---------------- Date & Utility Functions ---------------- */

    const padNumber = (num) => String(num).padStart(4, '0');

    // Exactly 3 characters for every month as requested (Jan, Feb, Mar, Apr, May, Jun, Jul, Aug, Sep, Oct, Nov, Dec)
    const monthShortNames = [
        'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
        'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
    ];

    const monthMap = {
        'jan': 0, 'january': 0,
        'feb': 1, 'february': 1,
        'mar': 2, 'march': 2,
        'apr': 3, 'april': 3,
        'may': 4,
        'jun': 5, 'june': 5,
        'jul': 6, 'july': 6,
        'aug': 7, 'august': 7,
        'sep': 8, 'sept': 8, 'september': 8,
        'oct': 9, 'october': 9,
        'nov': 10, 'november': 10,
        'dec': 11, 'december': 11
    };

    function parseDateValue(str) {
        if (!str) return new Date();
        if (str instanceof Date) return str;
        const s = String(str).trim();
        const parts = s.split('-');
        if (parts.length === 3) {
            if (parts[0].length === 4) { // YYYY-MM-DD
                const y = parseInt(parts[0], 10);
                const m = parseInt(parts[1], 10) - 1;
                const d = parseInt(parts[2], 10);
                if (!isNaN(y) && !isNaN(m) && !isNaN(d)) return new Date(y, m, d);
            }
            if (parts[2].length === 4) { // DD-MMM-YYYY or D-MMM-YYYY (e.g. 16-Sep-2026)
                const d = parseInt(parts[0], 10);
                const mKey = parts[1].toLowerCase();
                const y = parseInt(parts[2], 10);
                const m = monthMap[mKey] !== undefined ? monthMap[mKey] : (parseInt(parts[1], 10) - 1);
                if (!isNaN(y) && m !== undefined && !isNaN(m) && !isNaN(d)) return new Date(y, m, d);
            }
        }
        const fallback = new Date(s);
        return isNaN(fallback.getTime()) ? new Date() : fallback;
    }

    function formatDisplayDate(dateInput) {
        if (!dateInput) return '';
        const d = parseDateValue(dateInput);
        if (!d || isNaN(d.getTime())) return dateInput;
        const day = String(d.getDate()).padStart(2, '0');
        const month = monthShortNames[d.getMonth()];
        const year = d.getFullYear();
        return `${day}-${month}-${year}`;
    }

    function toISODate(dateInput) {
        const d = parseDateValue(dateInput);
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
    }

    const formatDate = (dateStr) => formatDisplayDate(dateStr);

    const addDaysToDate = (dateStr, days) => {
        try {
            const d = parseDateValue(dateStr);
            d.setDate(d.getDate() + days);
            return toISODate(d);
        } catch (e) {
            return dateStr;
        }
    };

    // Initialize invoice date field with today's formatted date
    if (inputs.invoiceDate && !inputs.invoiceDate.value) {
        inputs.invoiceDate.value = formatDisplayDate(new Date());
    }

    const showToast = (message, type = 'info') => {
        if (!toastContainer) return;
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        let iconHtml = '<i class="ph ph-info"></i>';
        if (type === 'success') iconHtml = '<i class="ph ph-check-circle"></i>';
        if (type === 'warn') iconHtml = '<i class="ph ph-warning-circle"></i>';
        if (type === 'error') iconHtml = '<i class="ph ph-x-circle"></i>';
        toast.innerHTML = `<span class="toast-icon">${iconHtml}</span><span class="toast-message">${message}</span>`;
        toastContainer.appendChild(toast);
        setTimeout(() => {
            if (toast.parentNode) toast.parentNode.removeChild(toast);
        }, 3200);
    };

    window.showToast = showToast;

    /* ---------------- Zoom & Scale Controls ---------------- */

    /* ---------------- Zoom & Scale Controls ---------------- */

    let isMobileZoomUserOverridden = false;
    let mobileZoomLevel = 1.0; // 1.0 = 100% (fitted to mobile screen). Minimum is always 1.0!
    let mobilePanX = 0;
    let mobilePanY = 0;

    const getBaseMobileFitScale = () => {
        const screenW = window.innerWidth;
        const targetW = Math.min(screenW - 20, 500);
        return targetW / 1080;
    };

    const applyMobilePreviewZoom = () => {
        if (!previewWrapper || !invoicePreview) return;
        if (window.innerWidth > 768) return;

        const baseFit = getBaseMobileFitScale();
        // Clamped minimum is 100% (1.0), maximum is 250% (2.5)
        mobileZoomLevel = Math.max(1.0, Math.min(2.5, mobileZoomLevel));
        const effectiveScale = baseFit * mobileZoomLevel;

        const baseWidth = Math.round(1080 * baseFit);
        const paperHeight = invoicePreview.offsetHeight || 1400;
        const baseHeight = Math.round(paperHeight * baseFit);

        previewWrapper.style.transform = 'none';
        previewWrapper.style.width = `${baseWidth}px`;
        previewWrapper.style.maxWidth = 'calc(100vw - 20px)';
        previewWrapper.style.height = `${baseHeight}px`;
        previewWrapper.style.minHeight = `${baseHeight}px`;
        previewWrapper.style.maxHeight = `${baseHeight}px`;
        previewWrapper.style.margin = '0 auto 20px';
        previewWrapper.style.overflow = 'hidden';
        previewWrapper.style.position = 'relative';

        // Clamp pan so invoice cannot be panned beyond view bounds
        const maxPanX = Math.round(baseWidth * (mobileZoomLevel - 1.0));
        const maxPanY = Math.round(baseHeight * (mobileZoomLevel - 1.0));

        if (mobileZoomLevel <= 1.002) {
            mobilePanX = 0;
            mobilePanY = 0;
        } else {
            mobilePanX = Math.min(0, Math.max(-maxPanX, mobilePanX));
            mobilePanY = Math.min(0, Math.max(-maxPanY, mobilePanY));
        }

        invoicePreview.style.transformOrigin = 'top left';
        invoicePreview.style.transform = `translate3d(${mobilePanX}px, ${mobilePanY}px, 0) scale(${effectiveScale})`;

        if (zoomLevelEl) {
            zoomLevelEl.textContent = `${Math.round(mobileZoomLevel * 100)}%`;
        }
    };

    const setZoom = (scale) => {
        const isMobile = window.innerWidth <= 768;
        if (isMobile) {
            isMobileZoomUserOverridden = true;
            mobileZoomLevel = Math.max(1.0, Math.min(2.5, scale));
            applyMobilePreviewZoom();
        } else {
            isMobileZoomUserOverridden = false;
            currentZoom = Math.min(Math.max(scale, 0.4), 1.8);
            if (previewWrapper) {
                previewWrapper.style.transform = `scale(${currentZoom})`;
                previewWrapper.style.height = '';
                previewWrapper.style.minHeight = '';
                previewWrapper.style.maxHeight = '';
                previewWrapper.style.width = '';
                previewWrapper.style.maxWidth = '';
                previewWrapper.style.margin = '';
                previewWrapper.style.overflow = '';
            }
            if (invoicePreview) {
                invoicePreview.style.transform = '';
                invoicePreview.style.transformOrigin = '';
            }
            if (zoomLevelEl) zoomLevelEl.textContent = `${Math.round(currentZoom * 100)}%`;
        }
    };

    const autoFitZoom = () => {
        const isMobile = window.innerWidth <= 768;
        if (isMobile) {
            isMobileZoomUserOverridden = false;
            mobileZoomLevel = 1.0;
            mobilePanX = 0;
            mobilePanY = 0;
            applyMobilePreviewZoom();
        } else {
            isMobileZoomUserOverridden = false;
            if (invoicePreview) {
                invoicePreview.style.transform = '';
                invoicePreview.style.transformOrigin = '';
            }
            if (previewWrapper) {
                previewWrapper.style.width = '';
                previewWrapper.style.maxWidth = '';
                previewWrapper.style.height = '';
                previewWrapper.style.minHeight = '';
                previewWrapper.style.maxHeight = '';
                previewWrapper.style.margin = '';
                previewWrapper.style.overflow = '';
            }
            const previewArea = document.querySelector('.preview-area');
            if (!previewArea) return;
            const availableWidth = previewArea.clientWidth - 80;
            const targetWidth = 1080;
            if (availableWidth < targetWidth) {
                const calculatedScale = Math.max(0.4, (availableWidth / targetWidth) * 0.98);
                setZoom(calculatedScale);
            } else {
                setZoom(1.0);
            }
        }
    };

    const updateMobilePreviewDimensions = () => {
        if (window.innerWidth <= 768) {
            applyMobilePreviewZoom();
        }
    };

    if (zoomInBtn) zoomInBtn.addEventListener('click', () => setZoom((window.innerWidth <= 768 ? mobileZoomLevel : currentZoom) + 0.1));
    if (zoomOutBtn) zoomOutBtn.addEventListener('click', () => setZoom((window.innerWidth <= 768 ? mobileZoomLevel : currentZoom) - 0.1));
    if (zoomResetBtn) zoomResetBtn.addEventListener('click', () => setZoom(1.0));
    if (zoomFitBtn) zoomFitBtn.addEventListener('click', autoFitZoom);
    window.addEventListener('resize', autoFitZoom);
    window.addEventListener('orientationchange', () => setTimeout(autoFitZoom, 150));

    // Two-finger pinch zoom centered on focal point & pan AND single-finger pan when zoomed in
    let initialPinchDistance = 0;
    let initialPinchZoom = 1.0;
    let initialPanX = 0;
    let initialPanY = 0;
    let initialMidX = 0;
    let initialMidY = 0;
    let initialFocalX = 0;
    let initialFocalY = 0;
    let isPinching = false;
    let singleTouchStartX = 0;
    let singleTouchStartY = 0;
    let isSinglePanning = false;

    if (previewWrapper) {
        const initPinch = (t1, t2) => {
            isPinching = true;
            isSinglePanning = false;
            initialPinchDistance = Math.hypot(t2.clientX - t1.clientX, t2.clientY - t1.clientY);
            initialPinchZoom = mobileZoomLevel;
            initialPanX = mobilePanX;
            initialPanY = mobilePanY;
            initialMidX = (t1.clientX + t2.clientX) / 2;
            initialMidY = (t1.clientY + t2.clientY) / 2;
            const rect = previewWrapper.getBoundingClientRect();
            initialFocalX = initialMidX - rect.left;
            initialFocalY = initialMidY - rect.top;
        };

        previewWrapper.addEventListener('touchstart', (e) => {
            if (window.innerWidth > 768) return;
            if (e.touches.length === 2) {
                initPinch(e.touches[0], e.touches[1]);
            } else if (e.touches.length === 1 && mobileZoomLevel > 1.01) {
                isSinglePanning = true;
                isPinching = false;
                singleTouchStartX = e.touches[0].clientX;
                singleTouchStartY = e.touches[0].clientY;
            }
        }, { passive: true });

        previewWrapper.addEventListener('touchmove', (e) => {
            if (window.innerWidth > 768) return;
            if (e.touches.length === 2) {
                const t1 = e.touches[0];
                const t2 = e.touches[1];
                const currentDistance = Math.hypot(t2.clientX - t1.clientX, t2.clientY - t1.clientY);

                if (!isPinching || initialPinchDistance <= 0) {
                    initPinch(t1, t2);
                    return;
                }

                if (e.cancelable) e.preventDefault(); // Prevent browser native viewport zoom

                const factor = currentDistance / initialPinchDistance;
                // Minimum zoom is strictly 100% (1.0), maximum is 250% (2.5)
                const targetZoom = Math.max(1.0, Math.min(2.5, initialPinchZoom * factor));

                const currentMidX = (t1.clientX + t2.clientX) / 2;
                const currentMidY = (t1.clientY + t2.clientY) / 2;
                const shiftX = currentMidX - initialMidX;
                const shiftY = currentMidY - initialMidY;

                // Direct focal point zoom: anchors precisely to the point under fingers
                const zoomRatio = targetZoom / initialPinchZoom;
                mobilePanX = (initialFocalX + shiftX) - (initialFocalX - initialPanX) * zoomRatio;
                mobilePanY = (initialFocalY + shiftY) - (initialFocalY - initialPanY) * zoomRatio;
                mobileZoomLevel = targetZoom;

                isMobileZoomUserOverridden = true;
                applyMobilePreviewZoom();
            } else if (e.touches.length === 1 && isSinglePanning && mobileZoomLevel > 1.01) {
                const curX = e.touches[0].clientX;
                const curY = e.touches[0].clientY;
                const deltaX = curX - singleTouchStartX;
                const deltaY = curY - singleTouchStartY;

                // Allow smooth 2D panning inside the zoomed invoice preview
                if (Math.abs(deltaX) > Math.abs(deltaY) || Math.abs(deltaX) > 4) {
                    if (e.cancelable) e.preventDefault();
                }

                mobilePanX += deltaX;
                mobilePanY += deltaY;
                singleTouchStartX = curX;
                singleTouchStartY = curY;
                applyMobilePreviewZoom();
            }
        }, { passive: false });

        const endTouch = (e) => {
            if (e.touches.length < 2) {
                isPinching = false;
                initialPinchDistance = 0;
            }
            if (e.touches.length === 1 && mobileZoomLevel > 1.01) {
                isSinglePanning = true;
                singleTouchStartX = e.touches[0].clientX;
                singleTouchStartY = e.touches[0].clientY;
            } else if (e.touches.length === 0) {
                isSinglePanning = false;
            }
        };

        previewWrapper.addEventListener('touchend', endTouch, { passive: true });
        previewWrapper.addEventListener('touchcancel', endTouch, { passive: true });
    }

    // Prevent accidental outer-page pinch zoom on mobile devices
    document.addEventListener('touchmove', (e) => {
        if (e.touches.length > 1 && !e.target.closest('.preview-wrapper')) {
            if (e.cancelable) e.preventDefault();
        }
    }, { passive: false });

    /* ---------------- Line Items Handlers ---------------- */

    function renderItemsInput() {
        itemsContainer.innerHTML = '';
        items.forEach((item, index) => {
            const row = document.createElement('div');
            row.className = 'item-row';
            row.innerHTML = `
                <div class="input-group-full">
                    <textarea placeholder="Description" oninput="updateItem(${index}, 'description', this.value)">${item.description || ''}</textarea>
                </div>
                <div class="input-group-full">
                    <textarea placeholder="Sub-detail (Line breaks supported)" style="font-size: 12px; color: #aaa; min-height: 54px;" oninput="updateItem(${index}, 'sub', this.value)">${item.sub || ''}</textarea>
                </div>
                <div class="input-row-flex">
                    <div class="input-group" style="flex: 1;">
                        <input type="number" placeholder="Price" value="${item.price || 0}" step="any" oninput="updateItem(${index}, 'price', this.value)">
                    </div>
                    <div class="item-remove-box" title="Remove line item">
                        <button class="item-remove" onclick="removeItem(${index})" title="Remove item"><i class="ph ph-x"></i></button>
                    </div>
                </div>
            `;
            itemsContainer.appendChild(row);
        });
        if (itemsCountBadge) {
            itemsCountBadge.textContent = `${items.length} ${items.length === 1 ? 'item' : 'items'}`;
        }
    }

    window.updateItem = (index, field, value) => {
        if (!items[index]) return;
        items[index][field] = field === 'price' ? (parseFloat(value) || 0) : value;
        userModifiedItems = true;
        updatePreview();
    };

    window.removeItem = (index) => {
        items.splice(index, 1);
        if (items.length === 0) {
            items.push({ description: '', sub: '', price: 0 });
        }
        userModifiedItems = true;
        renderItemsInput();
        updatePreview();
    };

    addItemBtn.addEventListener('click', () => {
        items.push({ description: 'New Item', sub: '', price: 0 });
        userModifiedItems = true;
        renderItemsInput();
        updatePreview();
    });

    /* ---------------- Live Preview Synchronizer ---------------- */

    let previewRafId = null;

    function updatePreview() {
        if (previewRafId) cancelAnimationFrame(previewRafId);
        previewRafId = requestAnimationFrame(() => {
            performPreviewUpdate();
            previewRafId = null;
        });
    }

    function updatePreviewImmediate() {
        if (previewRafId) {
            cancelAnimationFrame(previewRafId);
            previewRafId = null;
        }
        performPreviewUpdate();
    }

    function performPreviewUpdate() {
        const invNo = inputs.invoiceNumber.value.trim() || '0001';
        previews.number.textContent = invNo;
        previews.date.textContent = formatDate(inputs.invoiceDate.value);

        // "From" information automatically pulled from Settings / Profile
        const profile = InvoiceDB.getProfile();
        const fromFullName = `${profile.firstName || ''} ${profile.lastName || ''}`.trim() || 'Your Name';
        previews.fromName.textContent = fromFullName;
        previews.fromLocation.textContent = profile.address || '';
        previews.fromEmail.textContent = profile.email || '';

        // "To" client information automatically pulled from selected Client
        const toName = currentClient.name || 'Client Name';
        previews.toName.textContent = toName;
        previews.clientNameHeader.textContent = toName;
        previews.toLocation.textContent = currentClient.location || '';
        previews.toEmail.textContent = currentClient.email || '';

        // Visibility for party details
        [previews.fromName, previews.fromLocation, previews.fromEmail,
        previews.toName, previews.toLocation, previews.toEmail].forEach(el => {
            if (el.textContent.trim() === '') el.classList.add('hidden');
            else el.classList.remove('hidden');
        });

        // Line Items Table
        previews.itemsList.innerHTML = '';
        let total = 0;
        items.forEach(item => {
            total += (item.price || 0);
            const itemDiv = document.createElement('div');
            itemDiv.className = 'item-row-display';
            itemDiv.innerHTML = `
                <div class="item-desc-group">
                    <div class="item-name">${item.description || ''}</div>
                    <div class="item-link">${item.sub ? item.sub.replace(/\n/g, '<br>') : ''}</div>
                </div>
                <div class="col-divider"></div>
                <div class="item-price">
                    <span>${getCurrencySymbol()}</span><span>${(item.price || 0).toFixed(2)}</span>
                </div>
            `;
            previews.itemsList.appendChild(itemDiv);
        });

        previews.total.textContent = total.toFixed(2);

        // Payment Breakdown
        const showAdvance = inputs.toggleAdvance.checked;
        const showBalance = inputs.toggleBalance.checked;
        const breakdownContainer = previews.breakdownContainer || document.querySelector('.invoice-payment-breakdown');

        if (showAdvance) {
            advanceInputGroup.classList.remove('hidden');
            previews.rowAdvance.classList.remove('hidden');
        } else {
            advanceInputGroup.classList.add('hidden');
            previews.rowAdvance.classList.add('hidden');
        }

        if (showBalance) {
            previews.rowBalance.classList.remove('hidden');
        } else {
            previews.rowBalance.classList.add('hidden');
        }

        if (!showAdvance && !showBalance) {
            breakdownContainer.classList.add('hidden');
        } else {
            breakdownContainer.classList.remove('hidden');
            breakdownContainer.classList.toggle('has-both', showAdvance && showBalance);
        }

        const advanceVal = showAdvance ? (parseFloat(inputs.advanceAmount.value) || 0) : 0;
        const balanceVal = total - advanceVal;

        previews.advance.textContent = advanceVal.toFixed(2);
        previews.balance.textContent = balanceVal.toFixed(2);

        // Note: pulled automatically from Settings
        const settings = InvoiceDB.getSettings();
        previews.noteText.textContent = settings.defaultNote || 'Please complete your payment within 14 days, your cooperation is greatly appreciated.';

        // Toolbar Status & Title (if present)
        if (previews.statusPill) {
            previews.statusPill.textContent = currentInvoiceStatus;
            previews.statusPill.className = `status-pill status-${currentInvoiceStatus.toLowerCase()}`;
        }
        if (previews.toolbarDocTitle) {
            previews.toolbarDocTitle.innerHTML = `<i class="ph ph-receipt"></i> Invoice #${invNo}`;
        }

        // Platform & Bank Visibility
        updatePlatformAndBankVisibility();

        // Update mobile wrapper height to match scaled preview
        updateMobilePreviewDimensions();
    }

    function updatePlatformAndBankVisibility() {
        const updatePlatform = (id) => {
            const nameEl = document.getElementById(`preview-${id}-name`);
            const emailEl = document.getElementById(`preview-${id}-email`);
            const container = document.getElementById(`preview-col-${id}`);
            const emailRow = document.getElementById(`preview-row-${id}-email`);

            const name = nameEl ? nameEl.textContent.trim() : '';
            const email = emailEl ? emailEl.textContent.trim() : '';

            if (!name && !email) {
                if (container) container.classList.add('hidden');
            } else {
                if (container) container.classList.remove('hidden');
                if (emailRow) {
                    if (!email) emailRow.classList.add('hidden');
                    else emailRow.classList.remove('hidden');
                }
            }
        };

        updatePlatform('platform1');
        updatePlatform('platform2');

        const paymentMethods = document.querySelector('.payment-methods');
        const bankDetails = document.querySelector('.bank-details');
        const paymentInfoTitle = document.querySelector('.invoice-footer-info h3');
        const pMethodsVisible = Array.from(paymentMethods.querySelectorAll('.col:not(.hidden)')).length > 0;

        const bankRows = [
            'first-name', 'last-name', 'address', 'account-number',
            'bank-name', 'branch-name', 'branch-code', 'swift-code', 'routing-no'
        ];
        let anyBankVisible = false;
        bankRows.forEach(id => {
            const valEl = document.getElementById(`preview-${id}`);
            const rowEl = document.getElementById(`preview-row-${id}`);
            if (valEl && rowEl) {
                if (valEl.textContent.trim() === '') {
                    rowEl.classList.add('hidden');
                } else {
                    rowEl.classList.remove('hidden');
                    anyBankVisible = true;
                }
            }
        });

        if (bankDetails) {
            if (!anyBankVisible) bankDetails.classList.add('hidden');
            else bankDetails.classList.remove('hidden');
        }

        if (paymentInfoTitle) {
            if (!pMethodsVisible && !anyBankVisible) paymentInfoTitle.classList.add('hidden');
            else paymentInfoTitle.classList.remove('hidden');
        }

        // Signature and mobile divider logic
        const mobileVal = previews.mobile ? previews.mobile.textContent.trim() : '';
        const fullName = previews.signatureName ? previews.signatureName.textContent.trim() : '';
        if (previews.mobileContainer) {
            if (mobileVal) previews.mobileContainer.classList.remove('hidden');
            else previews.mobileContainer.classList.add('hidden');
        }
        if (previews.mobileDivider) {
            if (mobileVal && fullName) previews.mobileDivider.classList.remove('hidden');
            else previews.mobileDivider.classList.add('hidden');
        }
    }

    // Attach listeners to sidebar inputs
    Object.values(inputs).forEach(input => {
        if (input) {
            input.addEventListener('input', updatePreview);
            if (input.tagName === 'SELECT' || input.type === 'checkbox') {
                input.addEventListener('change', updatePreview);
            }
        }
    });

    /* ---------------- Reusable Custom Date Picker Factory ---------------- */

    const monthNames = [
        'January', 'February', 'March', 'April', 'May', 'June',
        'July', 'August', 'September', 'October', 'November', 'December'
    ];

    /**
     * createDatePicker(config) — Reusable date picker factory.
     * @param {object} config
     *   containerId   {string}   ID of the .custom-datepicker-container div
     *   popupId       {string}   ID of the .datepicker-popup div
     *   inputId       {string}   ID of the text input to display the date
     *   gridId        {string}   ID of the .cal-days-grid div
     *   monthYearId   {string}   ID of the .cal-month-year display div
     *   prevBtnId     {string}   ID of the prev-month button
     *   nextBtnId     {string}   ID of the next-month button
     *   todayBtnId    {string}   ID of the Today/Set Today button
     *   closeBtnId    {string}   ID of the Close/Clear button
     *   onSelect      {function} Called with (isoDateStr) when a date is picked
     *   onClear       {function} Called when the close/clear button is clicked (optional)
     *   getSelected   {function} Returns current ISO date string for highlighting
     */
    function createDatePicker(config) {
        const container = document.getElementById(config.containerId);
        const popup     = document.getElementById(config.popupId);
        const input     = document.getElementById(config.inputId);
        const grid      = document.getElementById(config.gridId);
        const monthYearEl = document.getElementById(config.monthYearId);
        const prevBtn   = document.getElementById(config.prevBtnId);
        const nextBtn   = document.getElementById(config.nextBtnId);
        const todayBtn  = document.getElementById(config.todayBtnId);
        const closeBtn  = document.getElementById(config.closeBtnId);

        if (!container || !popup) return null;

        let calYear  = new Date().getFullYear();
        let calMonth = new Date().getMonth();

        function renderGrid() {
            if (!grid || !monthYearEl) return;
            grid.innerHTML = '';
            monthYearEl.textContent = `${monthNames[calMonth]} ${calYear}`;

            const selectedDateStr = config.getSelected ? config.getSelected() : '';
            const now = new Date();
            const todayStr = `${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')}`;

            const firstDayIndex = new Date(calYear, calMonth, 1).getDay();
            const daysInMonth   = new Date(calYear, calMonth + 1, 0).getDate();
            const daysInPrevMonth = new Date(calYear, calMonth, 0).getDate();

            for (let i = firstDayIndex - 1; i >= 0; i--) {
                const dayNum  = daysInPrevMonth - i;
                const pm = calMonth === 0 ? 11 : calMonth - 1;
                const py = calMonth === 0 ? calYear - 1 : calYear;
                const ds = `${py}-${String(pm+1).padStart(2,'0')}-${String(dayNum).padStart(2,'0')}`;
                const cell = makeCell(dayNum, ds, 'other-month', selectedDateStr, todayStr);
                grid.appendChild(cell);
            }
            for (let d = 1; d <= daysInMonth; d++) {
                const ds = `${calYear}-${String(calMonth+1).padStart(2,'0')}-${String(d).padStart(2,'0')}`;
                const cell = makeCell(d, ds, '', selectedDateStr, todayStr);
                grid.appendChild(cell);
            }
            const totalCells = firstDayIndex + daysInMonth;
            const rem = (totalCells > 35 ? 42 : 35) - totalCells;
            for (let d = 1; d <= rem; d++) {
                const nm = calMonth === 11 ? 0 : calMonth + 1;
                const ny = calMonth === 11 ? calYear + 1 : calYear;
                const ds = `${ny}-${String(nm+1).padStart(2,'0')}-${String(d).padStart(2,'0')}`;
                const cell = makeCell(d, ds, 'other-month', selectedDateStr, todayStr);
                grid.appendChild(cell);
            }
        }

        function makeCell(day, dateStr, extraClass, selectedStr, todayStr) {
            const cell = document.createElement('button');
            cell.type = 'button';
            cell.className = 'cal-day' + (extraClass ? ' ' + extraClass : '');
            cell.textContent = day;
            cell.dataset.date = dateStr;
            if (dateStr === selectedStr) cell.classList.add('selected');
            if (dateStr === todayStr)    cell.classList.add('today');
            return cell;
        }

        function open() {
            // Close all other open pickers first
            document.querySelectorAll('.datepicker-popup:not(.hidden)').forEach(p => {
                if (p !== popup) {
                    p.classList.add('hidden');
                    p.closest('.custom-datepicker-container')?.classList.remove('is-open');
                }
            });
            const selected = config.getSelected ? config.getSelected() : '';
            if (selected) {
                const d = parseDateValue(selected);
                calYear  = d.getFullYear();
                calMonth = d.getMonth();
            } else {
                calYear  = new Date().getFullYear();
                calMonth = new Date().getMonth();
            }
            renderGrid();
            popup.classList.remove('hidden');
            container.classList.add('is-open');
        }

        function close() {
            popup.classList.add('hidden');
            container.classList.remove('is-open');
        }

        function toggle() {
            popup.classList.contains('hidden') ? open() : close();
        }

        function selectDate(dateStr) {
            if (input) {
                input.value = formatDisplayDate(dateStr);
                input.dispatchEvent(new Event('input',  { bubbles: true }));
                input.dispatchEvent(new Event('change', { bubbles: true }));
            }
            if (config.onSelect) config.onSelect(dateStr);
            close();
        }

        // Wire up events
        if (input) input.addEventListener('click', (e) => { e.stopPropagation(); toggle(); });
        container.addEventListener('click', (e) => {
            if (popup.contains(e.target)) return;
            e.stopPropagation(); toggle();
        });

        if (prevBtn) prevBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            calMonth--;
            if (calMonth < 0) { calMonth = 11; calYear--; }
            renderGrid();
        });
        if (nextBtn) nextBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            calMonth++;
            if (calMonth > 11) { calMonth = 0; calYear++; }
            renderGrid();
        });
        if (grid) grid.addEventListener('click', (e) => {
            e.stopPropagation();
            const btn = e.target.closest('.cal-day');
            if (btn && btn.dataset.date) selectDate(btn.dataset.date);
        });
        if (todayBtn) todayBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            const now = new Date();
            selectDate(`${now.getFullYear()}-${String(now.getMonth()+1).padStart(2,'0')}-${String(now.getDate()).padStart(2,'0')}`);
        });
        if (closeBtn) closeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            if (config.onClear) {
                config.onClear();
            } else {
                close();
            }
        });

        // Return public API
        return { open, close, toggle, renderGrid, syncTo(dateStr) {
            const d = parseDateValue(dateStr);
            calYear  = d.getFullYear();
            calMonth = d.getMonth();
            renderGrid();
        }};
    }

    /* ---- Invoice Date Picker (main) ---- */
    const invoiceDatePicker = createDatePicker({
        containerId:  'customDatePicker',
        popupId:      'datepickerPopup',
        inputId:      'invoiceDate',
        gridId:       'calDaysGrid',
        monthYearId:  'calMonthYear',
        prevBtnId:    'calPrevMonth',
        nextBtnId:    'calNextMonth',
        todayBtnId:   'calTodayBtn',
        closeBtnId:   'calCloseBtn',
        getSelected:  () => inputs.invoiceDate ? toISODate(inputs.invoiceDate.value) : '',
        onSelect:     () => updatePreview()
    });

    window.customDatePickerSync = (dateStr) => {
        if (invoiceDatePicker) invoiceDatePicker.syncTo(dateStr);
    };

    /* ---- History From Date Picker ---- */
    createDatePicker({
        containerId:  'historyFromDatePicker',
        popupId:      'historyFromDatePopup',
        inputId:      'history-from-date',
        gridId:       'historyFromCalGrid',
        monthYearId:  'historyFromCalMonthYear',
        prevBtnId:    'historyFromCalPrev',
        nextBtnId:    'historyFromCalNext',
        todayBtnId:   'historyFromCalToday',
        closeBtnId:   'historyFromCalClose',
        getSelected:  () => {
            const el = document.getElementById('history-from-date');
            return el && el.value ? toISODate(el.value) : '';
        },
        onSelect:     () => updateEarningsDisplay(),
        onClear:      () => {
            const el = document.getElementById('history-from-date');
            if (el) { el.value = ''; el.dispatchEvent(new Event('input', { bubbles: true })); }
            document.getElementById('historyFromDatePopup')?.classList.add('hidden');
            document.getElementById('historyFromDatePicker')?.classList.remove('is-open');
            updateEarningsDisplay();
        }
    });

    /* ---- History To Date Picker ---- */
    createDatePicker({
        containerId:  'historyToDatePicker',
        popupId:      'historyToDatePopup',
        inputId:      'history-to-date',
        gridId:       'historyToCalGrid',
        monthYearId:  'historyToCalMonthYear',
        prevBtnId:    'historyToCalPrev',
        nextBtnId:    'historyToCalNext',
        todayBtnId:   'historyToCalToday',
        closeBtnId:   'historyToCalClose',
        getSelected:  () => {
            const el = document.getElementById('history-to-date');
            return el && el.value ? toISODate(el.value) : '';
        },
        onSelect:     () => updateEarningsDisplay(),
        onClear:      () => {
            const el = document.getElementById('history-to-date');
            if (el) { el.value = ''; el.dispatchEvent(new Event('input', { bubbles: true })); }
            document.getElementById('historyToDatePopup')?.classList.add('hidden');
            document.getElementById('historyToDatePicker')?.classList.remove('is-open');
            updateEarningsDisplay();
        }
    });

    // Global: close any open picker on outside click or Escape
    document.addEventListener('click', (e) => {
        document.querySelectorAll('.datepicker-popup:not(.hidden)').forEach(popup => {
            const cont = popup.closest('.custom-datepicker-container');
            if (cont && !cont.contains(e.target)) {
                popup.classList.add('hidden');
                cont.classList.remove('is-open');
            }
        });
    });
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            document.querySelectorAll('.datepicker-popup:not(.hidden)').forEach(popup => {
                popup.classList.add('hidden');
                popup.closest('.custom-datepicker-container')?.classList.remove('is-open');
            });
        }
    });

    /* ---------------- Client Management ---------------- */

    function populateClientDropdown() {
        const clients = InvoiceDB.getClients();
        inputs.clientSelect.innerHTML = '<option value="">Select a Client</option>';
        clients.forEach((client) => {
            const opt = document.createElement('option');
            opt.value = client.id;
            opt.textContent = client.name + (client.location ? ` (${client.location})` : '');
            if (currentClient && currentClient.id === client.id) {
                opt.selected = true;
            }
            inputs.clientSelect.appendChild(opt);
        });
        if (!currentClient || !currentClient.id) {
            inputs.clientSelect.value = '';
        }
        // Sync the custom select wrapper after options are rebuilt
        inputs.clientSelect.dispatchEvent(new Event('change', { bubbles: true }));
    }

    inputs.clientSelect.addEventListener('change', (e) => {
        const clientId = e.target.value;
        if (!clientId) {
            currentClient = { id: null, name: 'Client Name', location: 'City/State, Country', email: 'client@example.com', currency: 'USD', currencySymbol: '$' };
            applyClientCurrencyToPreview(currentClient);
            updatePreview();
            return;
        }
        const client = InvoiceDB.getClient(clientId);
        if (client) {
            currentClient = {
                id: client.id,
                name: client.name,
                location: client.location || '',
                email: client.email || '',
                currency: client.currency || 'USD',
                currencySymbol: client.currencySymbol || '$'
            };
            applyClientCurrencyToPreview(currentClient);
            updatePreview();
            showToast(`Selected client: ${client.name} (${client.currency || 'USD'})`, 'info');
        }
    });

    // Apply a client's currency to all preview currency symbols
    function applyClientCurrencyToPreview(client) {
        const sym = client.currencySymbol || '$';
        document.querySelectorAll('.preview-currency-symbol').forEach(el => {
            el.textContent = sym;
        });
    }

    // Client Management Tab Logic
    toggleAddClientBtn.addEventListener('click', () => {
        clientFormContainer.classList.toggle('hidden');
        if (!clientFormContainer.classList.contains('hidden')) {
            clientFormTitle.innerHTML = '<i class="ph ph-user-plus"></i> Add New Client';
            clientFormId.value = '';
            clientFormName.value = '';
            clientFormEmail.value = '';
            clientFormLocation.value = '';
            clientFormPhone.value = '';
            clientFormCurrency.value = 'USD';
            syncCustomSelect(clientFormCurrency);
            clientFormNotes.value = '';
            clientFormName.focus();
        }
    });

    cancelClientFormBtn.addEventListener('click', () => {
        clientFormContainer.classList.add('hidden');
    });

    saveClientFormBtn.addEventListener('click', async () => {
        const name     = clientFormName.value.trim();
        const email    = clientFormEmail.value.trim();
        const location = clientFormLocation.value.trim();

        // Validation helper: highlight field + shake, then focus
        function markInvalid(input, msg) {
            input.style.borderColor = 'var(--accent-color)';
            input.style.boxShadow   = '0 0 0 3px rgba(234,78,37,0.2)';
            input.classList.add('shake-error');
            input.addEventListener('input', () => {
                input.style.borderColor = '';
                input.style.boxShadow   = '';
                input.classList.remove('shake-error');
            }, { once: true });
            input.focus();
            input.scrollIntoView({ behavior: 'smooth', block: 'center' });
            showToast(msg, 'error');
        }

        if (!name) {
            markInvalid(clientFormName, 'Client / Company Name is required.');
            return;
        }
        if (!email) {
            markInvalid(clientFormEmail, 'Email Address is required.');
            return;
        }
        // Basic email format check
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!emailRegex.test(email)) {
            markInvalid(clientFormEmail, 'Please enter a valid email address.');
            return;
        }
        if (!location) {
            markInvalid(clientFormLocation, 'Location / City, Country is required.');
            return;
        }

        // Get selected currency data-symbol
        const currencyOption = clientFormCurrency.options[clientFormCurrency.selectedIndex];
        const clientData = {
            id: clientFormId.value || undefined,
            name: name,
            email: clientFormEmail.value.trim(),
            location: clientFormLocation.value.trim(),
            phone: clientFormPhone.value.trim(),
            notes: clientFormNotes.value.trim(),
            currency: clientFormCurrency.value || 'USD',
            currencySymbol: (currencyOption && currencyOption.getAttribute('data-symbol')) || '$'
        };

        const saved = await InvoiceDB.saveClient(clientData);
        clientFormContainer.classList.add('hidden');
        renderClientsModalList();
        populateClientDropdown();
        updateDBBadge();

        // If currently using this client, update live preview with new currency too
        if (currentClient && (currentClient.id === saved.id || currentClient.name === saved.name)) {
            currentClient = {
                id: saved.id,
                name: saved.name,
                location: saved.location || '',
                email: saved.email || '',
                currency: saved.currency || 'USD',
                currencySymbol: saved.currencySymbol || '$'
            };
            applyClientCurrencyToPreview(currentClient);
            updatePreview();
        }

        showToast(`Client "${name}" saved successfully!`, 'success');
    });

    function renderClientsModalList(searchQuery = '') {
        const clients = InvoiceDB.getClients();
        const q = searchQuery.toLowerCase().trim();
        const filtered = clients.filter(c =>
            c.name.toLowerCase().includes(q) ||
            (c.email && c.email.toLowerCase().includes(q)) ||
            (c.location && c.location.toLowerCase().includes(q))
        );

        modalClientsCount.textContent = `${clients.length} ${clients.length === 1 ? 'Client' : 'Clients'}`;
        clientsListContainer.innerHTML = '';

        if (filtered.length === 0) {
            clientsListContainer.innerHTML = `
                <div style="text-align: center; color: #888; padding: 40px 20px;">
                    ${q ? 'No clients match your search.' : 'No clients saved in database yet. Click "+ Add New Client" above.'}
                </div>
            `;
            return;
        }

        filtered.forEach(client => {
            const card = document.createElement('div');
            card.className = 'client-card';
            const currencyBadge = client.currency && client.currency !== 'USD'
                ? `<span class="client-currency-badge">${client.currency}</span>`
                : `<span class="client-currency-badge">USD</span>`;
            card.innerHTML = `
                <div class="client-card-info">
                    <h4>${client.name} ${currencyBadge}</h4>
                    <div class="client-card-meta">
                        ${client.email ? `<span title="${client.email}"><i class="ph ph-envelope-simple"></i> <span class="client-card-email">${client.email}</span></span>` : ''}
                        ${client.location ? `<span><i class="ph ph-map-pin"></i> ${client.location}</span>` : ''}
                        ${client.phone ? `<span><i class="ph ph-phone"></i> ${client.phone}</span>` : ''}
                    </div>
                    ${client.notes ? `<div class="client-card-notes"><i class="ph ph-note"></i> ${client.notes}</div>` : ''}
                </div>
                <div class="client-card-actions">
                    <button class="action-icon-btn" onclick="useClientInInvoice('${client.id}')" title="Use client in current invoice"><i class="ph ph-arrow-square-in"></i></button>
                    <button class="action-icon-btn" onclick="editClient('${client.id}')" title="Edit client details"><i class="ph ph-pencil-simple"></i></button>
                    <button class="action-icon-btn delete-btn" onclick="deleteClientHandler('${client.id}')" title="Delete client"><i class="ph ph-trash"></i></button>
                </div>
            `;
            clientsListContainer.appendChild(card);
        });
    }

    clientSearchInput.addEventListener('input', (e) => {
        renderClientsModalList(e.target.value);
    });

    window.useClientInInvoice = (clientId) => {
        const client = InvoiceDB.getClient(clientId);
        if (client) {
            currentClient = {
                id: client.id,
                name: client.name,
                location: client.location || '',
                email: client.email || '',
                currency: client.currency || 'USD',
                currencySymbol: client.currencySymbol || '$'
            };
            if (inputs.clientSelect) {
                inputs.clientSelect.value = client.id;
            }
            applyClientCurrencyToPreview(currentClient);
            updatePreview();
            switchTab('tab-invoice');
            showToast(`Loaded ${client.name} into invoice (${client.currency || 'USD'})`, 'success');
        }
    };

    window.editClient = (clientId) => {
        const client = InvoiceDB.getClient(clientId);
        if (!client) return;

        clientFormTitle.innerHTML = '<i class="ph ph-pencil-simple"></i> Edit Client';
        clientFormId.value = client.id;
        clientFormName.value = client.name || '';
        clientFormEmail.value = client.email || '';
        clientFormLocation.value = client.location || '';
        clientFormPhone.value = client.phone || '';
        clientFormCurrency.value = client.currency || 'USD';
        syncCustomSelect(clientFormCurrency);
        clientFormNotes.value = client.notes || '';

        clientFormContainer.classList.remove('hidden');
        clientFormContainer.scrollIntoView({ behavior: 'smooth' });
    };

    window.deleteClientHandler = async (clientId) => {
        const client = InvoiceDB.getClient(clientId);
        const name = client ? client.name : 'this client';
        if (!confirm(`Are you sure you want to delete ${name}?`)) return;

        await InvoiceDB.deleteClient(clientId);
        renderClientsModalList(clientSearchInput.value);
        populateClientDropdown();
        updateDBBadge();
        showToast(`Client deleted`, 'info');
    };

    /* ---------------- Invoice Lifecycle & Saving ---------------- */

    function validateInvoiceReady(actionName = 'saving') {
        // 1. Client selection validation: user must select a real client
        const selectedClientId = inputs.clientSelect ? inputs.clientSelect.value.trim() : '';
        const hasValidClient = Boolean(selectedClientId || (currentClient && currentClient.name && currentClient.name !== 'Client Name' && currentClient.name.trim() !== ''));

        if (!hasValidClient) {
            showToast(`Please select a client before ${actionName} the invoice.`, 'warn');
            if (inputs.clientSelect) {
                inputs.clientSelect.focus();
                inputs.clientSelect.style.borderColor = 'var(--accent-color)';
                setTimeout(() => {
                    if (inputs.clientSelect) inputs.clientSelect.style.borderColor = '';
                }, 2000);
            }
            return false;
        }

        // 2. Items validation: ensure user didn't leave items untouched with default dummy data or empty
        const isUntouchedDefault = !userModifiedItems && items.length === 1 &&
            items[0].description === 'Website Development' &&
            items[0].sub === 'https://websitename.com' &&
            items[0].price === 200;

        const hasRealItems = items && items.length > 0 && items.some(it => (it.description || '').trim() !== '');

        if (isUntouchedDefault || !hasRealItems) {
            showToast(`Please add or customize your invoice items before ${actionName} the invoice.`, 'warn');
            if (itemsContainer) {
                const firstInput = itemsContainer.querySelector('textarea, input');
                if (firstInput) {
                    firstInput.focus();
                    firstInput.style.borderColor = 'var(--accent-color)';
                    setTimeout(() => {
                        if (firstInput) firstInput.style.borderColor = '';
                    }, 2000);
                }
            }
            return false;
        }

        return true;
    }

    async function saveCurrentInvoice(notify = true) {
        updatePreviewImmediate();
        if (!validateInvoiceReady(notify ? 'saving' : 'downloading')) {
            return null;
        }
        let total = 0;
        items.forEach(item => total += (item.price || 0));
        const advanceVal = inputs.toggleAdvance.checked ? (parseFloat(inputs.advanceAmount.value) || 0) : 0;
        const balanceVal = total - advanceVal;

        const invNo = inputs.invoiceNumber.value.trim() || '0001';
        const profile = InvoiceDB.getProfile();
        const settings = InvoiceDB.getSettings();
        const fromFullName = `${profile.firstName || ''} ${profile.lastName || ''}`.trim() || 'Your Name';

        const invoiceData = {
            id: currentEditingInvoiceId || `inv_${Date.now()}`,
            invoiceNumber: invNo,
            date: toISODate(inputs.invoiceDate.value),
            dueDate: addDaysToDate(inputs.invoiceDate.value, settings.defaultDueDays || 14),
            status: currentInvoiceStatus || 'Pending',
            from: {
                name: fromFullName,
                location: profile.address || '',
                email: profile.email || ''
            },
            client: {
                id: currentClient.id || null,
                name: currentClient.name || 'Client Name',
                location: currentClient.location || '',
                email: currentClient.email || ''
            },
            items: JSON.parse(JSON.stringify(items)),
            toggleAdvance: inputs.toggleAdvance.checked,
            advanceAmount: advanceVal,
            toggleBalance: inputs.toggleBalance.checked,
            total: total,
            balance: balanceVal,
            note: settings.defaultNote || ''
        };

        const saved = await InvoiceDB.saveInvoice(invoiceData);
        currentEditingInvoiceId = saved.id;

        // Update editing chip in preview toolbar
        if (editingBanner) editingBanner.classList.add('hidden');
        if (previewEditingChip) previewEditingChip.classList.remove('hidden');

        updateDBBadge();
        updateEarningsDisplay();

        if (notify) {
            showToast(`Invoice #${saved.invoiceNumber} saved to JSON database!`, 'success');
        }
        return saved;
    }

    saveInvoiceBtn.addEventListener('click', () => saveCurrentInvoice(true));

    function getNextInvoiceNumber() {
        const invoices = InvoiceDB.getInvoices();
        const settings = InvoiceDB.getSettings();
        
        let maxNum = 0;
        if (invoices && invoices.length > 0) {
            invoices.forEach(inv => {
                const n = parseInt(inv.invoiceNumber, 10);
                if (!isNaN(n) && n > maxNum) maxNum = n;
            });
            const lastSetting = parseInt(settings.lastInvoiceNumber || '0', 10);
            if (lastSetting > maxNum) maxNum = lastSetting;
            return padNumber(maxNum + 1);
        } else {
            const lastSetting = parseInt(settings.lastInvoiceNumber || '0', 10);
            return padNumber(lastSetting > 0 ? lastSetting : 1);
        }
    }

    function startNewInvoice(silent = false) {
        currentEditingInvoiceId = null;
        if (editingBanner) editingBanner.classList.add('hidden');
        if (previewEditingChip) previewEditingChip.classList.add('hidden');
        currentInvoiceStatus = 'Pending';

        // Determine next invoice number (always brand new, never previous)
        const nextNum = getNextInvoiceNumber();
        inputs.invoiceNumber.value = nextNum;

        // Display with exactly 3-character month (e.g. 16-Sep-2026)
        const todayFormatted = formatDisplayDate(new Date());
        inputs.invoiceDate.value = todayFormatted;
        if (window.customDatePickerSync) {
            window.customDatePickerSync(todayFormatted);
        }

        // By default no client will be selected
        currentClient = { id: null, name: 'Client Name', location: 'City/State, Country', email: 'client@example.com' };
        if (inputs.clientSelect) {
            inputs.clientSelect.value = '';
        }

        items = [
            { description: 'Website Development', sub: 'https://websitename.com', price: 200 }
        ];

        inputs.toggleAdvance.checked = false;
        inputs.advanceAmount.value = 0;
        inputs.toggleBalance.checked = false;

        userModifiedItems = false;
        renderItemsInput();
        updatePreview();
        if (!silent) {
            showToast(`Started new Invoice #${nextNum}`, 'info');
        }
    }

    /* ---------------- Tab Switching Engine (Tabs instead of popups) ---------------- */

    function switchTab(tabKey) {
        if (activeTabKey === tabKey && tabPanes[tabKey]?.classList.contains('active')) {
            return;
        }

        if (tabTransitionTimeout) {
            clearTimeout(tabTransitionTimeout);
            tabTransitionTimeout = null;
        }

        const previousKey = activeTabKey;
        activeTabKey = tabKey;

        const tabList = [
            { key: 'tab-invoice', btn: tabBtnNew },
            { key: 'tab-history', btn: tabBtnHistory },
            { key: 'tab-clients', btn: tabBtnClients },
            { key: 'tab-settings', btn: tabBtnSettings }
        ];

        tabList.forEach(item => {
            if (item.btn) {
                const isSelected = (item.key === tabKey);
                item.btn.classList.toggle('active', isSelected);
                item.btn.setAttribute('aria-selected', isSelected ? 'true' : 'false');
            }
        });

        // Sidebar panel switching: Invoice controls vs Settings 6-digit sync
        const sidebarEl = document.querySelector('.sidebar');
        const invoiceControls = document.getElementById('sidebar-invoice-controls');
        const settingsPanel = document.getElementById('sidebar-settings-panel');

        if (sidebarEl) {
            if (tabKey === 'tab-settings') {
                // Do NOT blur sidebar on Settings tab! Show 6-digit sync panel
                sidebarEl.classList.remove('sidebar-blurred');
                if (invoiceControls) invoiceControls.classList.add('hidden');
                if (settingsPanel) settingsPanel.classList.remove('hidden');
            } else if (tabKey === 'tab-invoice') {
                sidebarEl.classList.remove('sidebar-blurred');
                if (invoiceControls) invoiceControls.classList.remove('hidden');
                if (settingsPanel) settingsPanel.classList.add('hidden');
            } else {
                sidebarEl.classList.add('sidebar-blurred');
                if (invoiceControls) invoiceControls.classList.remove('hidden');
                if (settingsPanel) settingsPanel.classList.add('hidden');
            }
        }

        const outgoingPane = tabPanes[previousKey];
        const incomingPane = tabPanes[tabKey];

        // Animate exit down, then enter from bottom to top
        if (outgoingPane && outgoingPane !== incomingPane && outgoingPane.classList.contains('active')) {
            outgoingPane.classList.add('tab-pane-exiting');

            tabTransitionTimeout = setTimeout(() => {
                tabTransitionTimeout = null;
                Object.keys(tabPanes).forEach(k => {
                    if (tabPanes[k]) {
                        tabPanes[k].classList.remove('active', 'tab-pane-exiting');
                    }
                });
                if (incomingPane) {
                    incomingPane.classList.add('active');
                }
                handleTabActivated(tabKey);
            }, 130);
        } else {
            Object.keys(tabPanes).forEach(k => {
                if (k !== tabKey && tabPanes[k]) {
                    tabPanes[k].classList.remove('active', 'tab-pane-exiting');
                }
            });
            if (incomingPane) {
                incomingPane.classList.add('active');
            }
            handleTabActivated(tabKey);
        }
    }

    function handleTabActivated(tabKey) {
        // Discard any unsaved settings edits when navigating between tabs
        loadProfileAndSettings();

        if (tabKey === 'tab-history') {
            updateEarningsDisplay();
        } else if (tabKey === 'tab-clients') {
            renderClientsModalList(clientSearchInput ? clientSearchInput.value : '');
        } else if (tabKey === 'tab-invoice') {
            autoFitZoom();
        }
    }

    window.switchAppTab = switchTab;

    if (tabBtnNew) {
        tabBtnNew.addEventListener('click', () => {
            if (activeTabKey !== 'tab-invoice') {
                switchTab('tab-invoice');
            } else {
                startNewInvoice(false);
            }
        });
    }

    if (tabBtnHistory) {
        tabBtnHistory.addEventListener('click', () => switchTab('tab-history'));
    }

    if (tabBtnClients) {
        tabBtnClients.addEventListener('click', () => switchTab('tab-clients'));
    }

    if (tabBtnSettings) {
        tabBtnSettings.addEventListener('click', () => switchTab('tab-settings'));
    }

    if (newInvoiceBtn && newInvoiceBtn !== tabBtnNew) {
        newInvoiceBtn.addEventListener('click', () => {
            switchTab('tab-invoice');
            startNewInvoice(false);
        });
    }

    if (cancelEditBtn) {
        cancelEditBtn.addEventListener('click', () => {
            switchTab('tab-invoice');
            startNewInvoice(false);
        });
    }

    if (cancelEditChipBtn) {
        cancelEditChipBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            switchTab('tab-invoice');
            startNewInvoice(false);
        });
    }

    window.loadInvoiceIntoEditor = (idOrNumber) => {
        const inv = InvoiceDB.getInvoice(idOrNumber);
        if (!inv) return;

        currentEditingInvoiceId = inv.id;
        inputs.invoiceNumber.value = inv.invoiceNumber || '0001';
        currentInvoiceStatus = inv.status || 'Pending';
        inputs.invoiceDate.value = formatDisplayDate(inv.date || new Date());
        if (window.customDatePickerSync) {
            window.customDatePickerSync(inv.date || inputs.invoiceDate.value);
        }

        const clientObj = inv.client || inv.to || {};
        currentClient = {
            id: clientObj.id || null,
            name: clientObj.name || 'Client Name',
            location: clientObj.location || '',
            email: clientObj.email || ''
        };

        // Match dropdown
        const matchedClient = InvoiceDB.getClients().find(c => (clientObj.id && c.id === clientObj.id) || c.name.toLowerCase() === (clientObj.name || '').toLowerCase());
        if (matchedClient) {
            inputs.clientSelect.value = matchedClient.id;
        } else {
            inputs.clientSelect.value = '';
        }

        items = Array.isArray(inv.items) && inv.items.length > 0
            ? JSON.parse(JSON.stringify(inv.items))
            : [{ description: 'Invoice Item', sub: '', price: inv.total || 0 }];
        userModifiedItems = true;

        inputs.toggleAdvance.checked = !!inv.toggleAdvance;
        inputs.advanceAmount.value = inv.advanceAmount || 0;
        inputs.toggleBalance.checked = !!inv.toggleBalance;

        // Show editing chip in preview toolbar
        if (editingBanner) editingBanner.classList.add('hidden');
        if (previewEditingChip) previewEditingChip.classList.remove('hidden');

        renderItemsInput();
        updatePreview();
        switchTab('tab-invoice');
        showToast(`Loaded Invoice #${inv.invoiceNumber} into editor`, 'info');
    };

    window.duplicateInvoice = (idOrNumber) => {
        const inv = InvoiceDB.getInvoice(idOrNumber);
        if (!inv) return;

        startNewInvoice();

        const clientObj = inv.client || inv.to || {};
        currentClient = {
            id: clientObj.id || null,
            name: clientObj.name || 'Client Name',
            location: clientObj.location || '',
            email: clientObj.email || ''
        };
        const matched = InvoiceDB.getClients().find(c => (clientObj.id && c.id === clientObj.id) || c.name.toLowerCase() === (clientObj.name || '').toLowerCase());
        if (matched && inputs.clientSelect) {
            inputs.clientSelect.value = matched.id;
        }

        items = JSON.parse(JSON.stringify(inv.items || []));
        inputs.toggleAdvance.checked = !!inv.toggleAdvance;
        inputs.advanceAmount.value = inv.advanceAmount || 0;
        inputs.toggleBalance.checked = !!inv.toggleBalance;
        currentInvoiceStatus = 'Draft';

        renderItemsInput();
        updatePreview();
        switchTab('tab-invoice');
        showToast(`Duplicated invoice as #${inputs.invoiceNumber.value} (Draft)`, 'success');
    };

    window.deleteInvoiceHandler = async (idOrNumber) => {
        const inv = InvoiceDB.getInvoice(idOrNumber);
        const invNo = inv ? inv.invoiceNumber : idOrNumber;
        if (!confirm(`Are you sure you want to delete Invoice #${invNo} from the database?`)) return;

        await InvoiceDB.deleteInvoice(idOrNumber);
        if (currentEditingInvoiceId === idOrNumber || inputs.invoiceNumber.value === invNo) {
            startNewInvoice();
        }
        updateEarningsDisplay();
        updateDBBadge();
        showToast(`Invoice #${invNo} deleted`, 'info');
    };

    window.changeInvoiceStatus = async (idOrNumber, newStatus) => {
        await InvoiceDB.updateInvoiceStatus(idOrNumber, newStatus);
        if (currentEditingInvoiceId === idOrNumber || inputs.invoiceNumber.value === idOrNumber) {
            currentInvoiceStatus = newStatus;
            updatePreview();
        }
        updateEarningsDisplay();
        showToast(`Status updated to ${newStatus}`, 'info');
    };

    /* ---------------- Print Engine (True Vector PDF) ---------------- */

    /* ---------------- Direct PDF Download Engine (Desktop & Mobile) ---------------- */

    const fallbackPrint = (invoiceNo, paperHeight, wrapper, originalPaper, origWrapperCss, origPaperCss) => {
        const height = paperHeight + 10;
        const width = 1080;

        const styleId = 'dynamic-print-settings';
        let styleTag = document.getElementById(styleId);
        if (!styleTag) {
            styleTag = document.createElement('style');
            styleTag.id = styleId;
            document.head.appendChild(styleTag);
        }

        styleTag.innerHTML = `
            @media print {
                @page {
                    size: ${width}px ${height}px;
                    margin: 0 !important;
                }
                html, body {
                    width: ${width}px !important;
                    min-width: ${width}px !important;
                    max-width: ${width}px !important;
                    height: auto !important;
                    margin: 0 auto !important;
                    padding: 0 !important;
                    overflow: visible !important;
                    background: #ffffff !important;
                    -webkit-print-color-adjust: exact !important;
                    print-color-adjust: exact !important;
                }
                .app-container, .app-body, .tab-content-area, #view-invoice, .preview-area {
                    display: block !important;
                    width: ${width}px !important;
                    min-width: ${width}px !important;
                    max-width: ${width}px !important;
                    height: auto !important;
                    min-height: 0 !important;
                    max-height: none !important;
                    overflow: visible !important;
                    margin: 0 auto !important;
                    padding: 0 !important;
                }
                .preview-wrapper {
                    display: block !important;
                    width: ${width}px !important;
                    min-width: ${width}px !important;
                    max-width: ${width}px !important;
                    height: auto !important;
                    min-height: 0 !important;
                    max-height: none !important;
                    overflow: visible !important;
                    margin: 0 auto !important;
                    padding: 0 !important;
                    border: none !important;
                    box-shadow: none !important;
                }
                .invoice-paper {
                    width: ${width}px !important;
                    min-width: ${width}px !important;
                    max-width: ${width}px !important;
                    height: auto !important;
                    min-height: ${paperHeight}px !important;
                    margin: 0 auto !important;
                    padding: 0 !important;
                    transform: none !important;
                    page-break-inside: avoid !important;
                    break-inside: avoid !important;
                }
            }
        `;

        const originalTitle = document.title;
        document.title = `Invoice - ${invoiceNo}`;

        let isCleanedUp = false;
        const cleanupPrint = () => {
            if (isCleanedUp) return;
            isCleanedUp = true;
            window.removeEventListener('afterprint', cleanupPrint);
            document.title = originalTitle;
            document.body.classList.remove('is-printing');
            if (wrapper) wrapper.style.cssText = origWrapperCss;
            originalPaper.style.cssText = origPaperCss;

            if (window.innerWidth <= 768 && typeof applyMobilePreviewZoom === 'function') {
                applyMobilePreviewZoom();
            }

            showToast(`Invoice #${invoiceNo} PDF downloaded!`, 'success');
        };

        window.addEventListener('afterprint', cleanupPrint, { once: true });
        setTimeout(cleanupPrint, 4000);

        window.print();
    };

    const triggerDownloadPdf = async () => {
        // Ensure invoice editor tab is active before downloading
        if (activeTabKey !== 'tab-invoice') {
            switchTab('tab-invoice');
        }

        // Validate client and items before proceeding with save or download
        if (!validateInvoiceReady('downloading')) {
            return;
        }

        // 1. Auto-save current invoice first
        const saved = await saveCurrentInvoice(false);
        if (!saved) return;

        const originalPaper = document.getElementById('invoice-preview');
        const wrapper = document.querySelector('.preview-wrapper');
        if (!originalPaper) return;

        const invoiceNo = inputs.invoiceNumber.value || '0001';

        // 2. Save original inline styles to restore after download
        const origWrapperCss = wrapper ? wrapper.style.cssText : '';
        const origPaperCss = originalPaper.style.cssText;

        showToast('Generating borderless PDF...', 'info');

        // Add printing preparation class to body
        document.body.classList.add('is-printing');

        // Fully unconstrain both wrapper and paper to full 1080px physical layout
        if (wrapper) {
            wrapper.style.cssText = 'transform: none !important; width: 1080px !important; min-width: 1080px !important; max-width: 1080px !important; height: auto !important; min-height: 0 !important; max-height: none !important; overflow: visible !important; margin: 0 !important; padding: 0 !important; box-shadow: none !important; border-radius: 0 !important;';
        }
        originalPaper.style.cssText = 'width: 1080px !important; min-width: 1080px !important; max-width: 1080px !important; height: auto !important; min-height: 0 !important; max-height: none !important; margin: 0 !important; transform: none !important; box-shadow: none !important;';

        await new Promise(r => setTimeout(r, 120));

        const paperHeight = Math.max(originalPaper.scrollHeight, originalPaper.offsetHeight, 1122);

        // Direct 1-Click PDF Download to Downloads folder across Desktop & Mobile
        if (typeof html2pdf !== 'undefined') {
            const opt = {
                margin: 0,
                filename: `Invoice-${invoiceNo}.pdf`,
                image: { type: 'jpeg', quality: 0.98 },
                html2canvas: {
                    scale: 2, // High resolution Retina clarity
                    useCORS: true,
                    logging: false,
                    width: 1080,
                    windowWidth: 1080,
                    scrollX: 0,
                    scrollY: 0
                },
                jsPDF: {
                    unit: 'px',
                    format: [1080, paperHeight + 10],
                    orientation: 'portrait',
                    hotfixes: ['px_scaling']
                }
            };

            try {
                // Generate the raw PDF Blob directly to bypass jsPDF's legacy "if (isSafari) window.open()" check
                const pdfBlob = await html2pdf().set(opt).from(originalPaper).outputPdf('blob');
                const filename = `Invoice-${invoiceNo}.pdf`;

                const isIOS = /iPhone|iPad|iPod/i.test(navigator.userAgent) || (navigator.platform === 'MacIntel' && navigator.maxTouchPoints > 1);
                const isSafari = /^((?!chrome|android).)*safari/i.test(navigator.userAgent);

                let savedDirectly = false;

                // On iOS Safari: use Web Share API so the user can immediately "Save to Files" without opening a new tab
                if (isIOS && isSafari && navigator.canShare) {
                    try {
                        const file = new File([pdfBlob], filename, { type: 'application/pdf' });
                        if (navigator.canShare({ files: [file] })) {
                            await navigator.share({
                                files: [file],
                                title: filename
                            });
                            savedDirectly = true;
                            showToast(`Invoice #${invoiceNo} ready!`, 'success');
                        }
                    } catch (shareErr) {
                        if (shareErr.name === 'AbortError') {
                            savedDirectly = true;
                        }
                    }
                }

                // If not shared (Desktop, Android, Edge, or fallback), trigger direct binary file download
                if (!savedDirectly) {
                    // Force application/octet-stream to prevent Safari from opening the PDF in a new viewer tab
                    const octetBlob = new Blob([pdfBlob], { type: 'application/octet-stream' });
                    const blobUrl = URL.createObjectURL(octetBlob);
                    const link = document.createElement('a');
                    link.href = blobUrl;
                    link.download = filename;
                    link.rel = 'noopener';
                    document.body.appendChild(link);
                    link.click();
                    setTimeout(() => {
                        if (document.body.contains(link)) document.body.removeChild(link);
                        URL.revokeObjectURL(blobUrl);
                    }, 1500);

                    showToast(`Invoice #${invoiceNo} downloaded to Downloads!`, 'success');
                }
            } catch (err) {
                console.error('Direct PDF export error, falling back to print:', err);
                fallbackPrint(invoiceNo, paperHeight, wrapper, originalPaper, origWrapperCss, origPaperCss);
                return;
            } finally {
                document.body.classList.remove('is-printing');
                if (wrapper) wrapper.style.cssText = origWrapperCss;
                originalPaper.style.cssText = origPaperCss;
                if (window.innerWidth <= 768 && typeof applyMobilePreviewZoom === 'function') {
                    applyMobilePreviewZoom();
                }
            }
        } else {
            fallbackPrint(invoiceNo, paperHeight, wrapper, originalPaper, origWrapperCss, origPaperCss);
        }
    };

    printBtn.addEventListener('click', triggerDownloadPdf);
    if (quickPrintBtn) quickPrintBtn.addEventListener('click', triggerDownloadPdf);

    /* ---------------- Earnings & History ---------------- */

    function updateEarningsDisplay() {
        const invoices = InvoiceDB.getInvoices();
        const fromDateISO = historyFromInput.value ? toISODate(historyFromInput.value) : '';
        const toDateISO   = historyToInput.value   ? toISODate(historyToInput.value)   : '';
        const statusFilter = historyStatusFilter ? historyStatusFilter.value : 'ALL';
        const query = (historySearchInput ? historySearchInput.value : '').toLowerCase().trim();

        let filtered = [...invoices];

        if (statusFilter && statusFilter !== 'ALL') {
            filtered = filtered.filter(i => (i.status || 'Pending').toLowerCase() === statusFilter.toLowerCase());
        }

        if (fromDateISO) {
            filtered = filtered.filter(i => {
                const invISO = toISODate(i.date || '');
                return invISO >= fromDateISO;
            });
        }

        if (toDateISO) {
            filtered = filtered.filter(i => {
                const invISO = toISODate(i.date || '');
                return invISO <= toDateISO;
            });
        }

        if (query) {
            filtered = filtered.filter(i => {
                const num = (i.invoiceNumber || '').toLowerCase();
                const client = (i.client?.name || i.to?.name || i.client || '').toLowerCase();
                return num.includes(query) || client.includes(query);
            });
        }

        // Sort descending by date & number
        filtered.sort((a, b) => new Date(b.date) - new Date(a.date) || b.invoiceNumber.localeCompare(a.invoiceNumber));

        // Compute Overview Stats
        let totalInvoiced = 0;
        let totalPaid = 0;
        let totalPending = 0;

        invoices.forEach(i => {
            const amt = i.total || 0;
            totalInvoiced += amt;
            const st = (i.status || 'Pending').toLowerCase();
            if (st === 'paid') totalPaid += amt;
            if (st === 'pending' || st === 'overdue') totalPending += amt;
        });

        statTotalInvoiced.textContent = `$${totalInvoiced.toFixed(2)}`;
        statTotalPaid.textContent = `$${totalPaid.toFixed(2)}`;
        statTotalPending.textContent = `$${totalPending.toFixed(2)}`;
        statInvoicesCount.textContent = invoices.length;

        // Render Invoices Table
        historyTableBody.innerHTML = '';
        if (filtered.length === 0) {
            historyTableBody.innerHTML = `
                <tr>
                    <td colspan="6" style="text-align: center; color: #888; padding: 36px 20px;">
                        No matching invoices found in database.
                    </td>
                </tr>
            `;
            return;
        }

        filtered.forEach(inv => {
            const clientName = inv.client?.name || inv.to?.name || inv.client || 'Client';
            const status = inv.status || 'Pending';
            const total = inv.total || 0;
            const row = document.createElement('tr');
            row.innerHTML = `
                <td class="history-invno-cell"><strong>#${inv.invoiceNumber}</strong></td>
                <td class="history-date-cell">${formatDate(inv.date)}</td>
                <td style="color: #fff; font-weight: 500;">${clientName}</td>
                <td style="color: var(--accent-color); font-weight: 600;">$${total.toFixed(2)}</td>
                <td>
                    <select class="table-status-select" onchange="changeInvoiceStatus('${inv.id}', this.value)">
                        <option value="Paid" ${status === 'Paid' ? 'selected' : ''}>Paid</option>
                        <option value="Pending" ${status === 'Pending' ? 'selected' : ''}>Pending</option>
                        <option value="Draft" ${status === 'Draft' ? 'selected' : ''}>Draft</option>
                        <option value="Overdue" ${status === 'Overdue' ? 'selected' : ''}>Overdue</option>
                    </select>
                </td>
                <td class="table-actions-cell">
                    <button class="action-icon-btn" onclick="loadInvoiceIntoEditor('${inv.id}')" title="Edit in invoice maker"><i class="ph ph-pencil-simple"></i></button>
                    <button class="action-icon-btn" onclick="duplicateInvoice('${inv.id}')" title="Duplicate invoice"><i class="ph ph-copy"></i></button>
                    <button class="action-icon-btn delete-btn" onclick="deleteInvoiceHandler('${inv.id}')" title="Delete invoice"><i class="ph ph-trash"></i></button>
                </td>
            `;
            historyTableBody.appendChild(row);
            // Init custom select for this row's status dropdown
            initCustomSelects(row);
        });
    }

    historySearchInput.addEventListener('input', updateEarningsDisplay);
    historyStatusFilter.addEventListener('change', updateEarningsDisplay);
    historyFromInput.addEventListener('input', updateEarningsDisplay);
    historyToInput.addEventListener('input', updateEarningsDisplay);

    exportCsvBtn.addEventListener('click', () => {
        try {
            const invoices = InvoiceDB.getInvoices();
            if (invoices.length === 0) {
                alert('No invoices available to export.');
                return;
            }
            InvoiceDB.exportInvoicesCSV(invoices);
            showToast('CSV export downloaded!', 'success');
        } catch (e) {
            alert(e.message);
        }
    });

    exportInvoicesJsonBtn.addEventListener('click', () => {
        try {
            InvoiceDB.exportInvoicesJSON();
            showToast('Invoices JSON exported!', 'success');
        } catch (e) {
            alert(e.message);
        }
    });

    /* ---------------- Settings & Profile Management ---------------- */

    const openSettingsTab = () => {
        switchTab('tab-settings');
    };

    if (dbBadge) dbBadge.addEventListener('click', openSettingsTab);

    function loadProfileAndSettings() {
        const profile = InvoiceDB.getProfile() || {};
        const settings = InvoiceDB.getSettings() || {};

        settingInputs.firstName.value = profile.firstName || '';
        settingInputs.lastName.value = profile.lastName || '';
        settingInputs.email.value = profile.email || '';
        settingInputs.mobile.value = profile.mobile || '';
        settingInputs.address.value = profile.address || '';

        const p1 = profile.platforms?.[0] || { name: 'PayPal', email: profile.email || '' };
        const p2 = profile.platforms?.[1] || { name: 'Wise', email: profile.email || '' };
        settingInputs.platform1Name.value = p1.name || 'PayPal';
        settingInputs.platform1Email.value = p1.email || '';
        settingInputs.platform2Name.value = p2.name || 'Wise';
        settingInputs.platform2Email.value = p2.email || '';

        const bank = profile.bank || {};
        settingInputs.bankName.value = bank.bankName || '';
        settingInputs.accountNumber.value = bank.accountNumber || '';
        settingInputs.branchName.value = bank.branchName || '';
        settingInputs.branchCode.value = bank.branchCode || '';
        settingInputs.swiftCode.value = bank.swiftCode || '';
        settingInputs.routingNo.value = bank.routingNo || '';

        settingInputs.defaultNote.value = settings.defaultNote || '';

        // Load currency selector
        if (settingInputs.currency) {
            settingInputs.currency.value = settings.currency || 'USD';
            syncCustomSelect(settingInputs.currency);
        }

        updateLogoDisplay(profile.companyLogo);
        syncProfileToPreviewsAndDefaults();
        updateDBBadge();
    }

    function syncProfileToPreviewsAndDefaults() {
        const profile = InvoiceDB.getProfile();
        const fullName = `${profile.firstName || ''} ${profile.lastName || ''}`.trim() || 'Your Name';

        // From details in preview
        if (previews.fromName) previews.fromName.textContent = fullName;
        if (previews.fromLocation) previews.fromLocation.textContent = profile.address || '';
        if (previews.fromEmail) previews.fromEmail.textContent = profile.email || '';

        // Preview Signature
        if (previews.signatureName) previews.signatureName.textContent = fullName;
        if (previews.mobile) previews.mobile.textContent = profile.mobile || '';

        // Platforms
        const p1 = profile.platforms?.[0] || { name: 'PayPal', email: '' };
        const p2 = profile.platforms?.[1] || { name: 'Wise', email: '' };

        const p1NameEl = document.getElementById('preview-platform1-name');
        const p1EmailEl = document.getElementById('preview-platform1-email');
        if (p1NameEl) p1NameEl.textContent = p1.name || 'PayPal';
        if (p1EmailEl) p1EmailEl.textContent = p1.email;

        const p2NameEl = document.getElementById('preview-platform2-name');
        const p2EmailEl = document.getElementById('preview-platform2-email');
        if (p2NameEl) p2NameEl.textContent = p2.name || 'Wise';
        if (p2EmailEl) p2EmailEl.textContent = p2.email;

        // Bank preview fields
        const bank = profile.bank || {};
        const bankMap = {
            'preview-first-name': profile.firstName || '',
            'preview-last-name': profile.lastName || '',
            'preview-address': profile.address || '',
            'preview-account-number': bank.accountNumber || '',
            'preview-bank-name': bank.bankName || '',
            'preview-branch-name': bank.branchName || '',
            'preview-branch-code': bank.branchCode || '',
            'preview-swift-code': bank.swiftCode || '',
            'preview-routing-no': bank.routingNo || ''
        };

        Object.keys(bankMap).forEach(id => {
            const el = document.getElementById(id);
            if (el) el.textContent = bankMap[id];
        });

        // Update currency symbols in preview
        const currencySymbol = getCurrencySymbol();
        document.querySelectorAll('.preview-currency-symbol').forEach(el => {
            el.textContent = currencySymbol;
        });

        updatePreview();
    }

    // Helper: get currency symbol — client-specific takes priority over global setting
    function getCurrencySymbol() {
        // 1. Active client's currency (per-client override)
        if (currentClient && currentClient.currencySymbol) {
            return currentClient.currencySymbol;
        }
        // 2. Global setting from the currency selector in Settings
        const settings = InvoiceDB.getSettings();
        const currencySelect = settingInputs.currency;
        if (currencySelect) {
            const selectedOption = currencySelect.options[currencySelect.selectedIndex];
            return (selectedOption && selectedOption.getAttribute('data-symbol')) || settings.currencySymbol || '$';
        }
        return settings.currencySymbol || '$';
    }


    // Save all settings explicitly from UI (Called ONLY on button click)
    async function saveAllSettingsFromUI(notify = false) {
        const data = InvoiceDB.getData() || {};
        data.isDemoData = false; // User has saved custom details (or cleared details)

        if (!data.profile) data.profile = {};
        if (!data.settings) data.settings = {};

        const profile = data.profile;
        const settings = data.settings;

        profile.firstName = settingInputs.firstName ? settingInputs.firstName.value.trim() : '';
        profile.lastName = settingInputs.lastName ? settingInputs.lastName.value.trim() : '';
        profile.email = settingInputs.email ? settingInputs.email.value.trim() : '';
        profile.mobile = settingInputs.mobile ? settingInputs.mobile.value.trim() : '';
        profile.address = settingInputs.address ? settingInputs.address.value.trim() : '';

        profile.platforms = [
            {
                id: 'platform1',
                name: settingInputs.platform1Name ? settingInputs.platform1Name.value.trim() || 'PayPal' : 'PayPal',
                email: settingInputs.platform1Email ? settingInputs.platform1Email.value.trim() : ''
            },
            {
                id: 'platform2',
                name: settingInputs.platform2Name ? settingInputs.platform2Name.value.trim() || 'Wise' : 'Wise',
                email: settingInputs.platform2Email ? settingInputs.platform2Email.value.trim() : ''
            }
        ];

        if (!profile.bank) profile.bank = {};
        profile.bank.bankName = settingInputs.bankName ? settingInputs.bankName.value.trim() : '';
        profile.bank.accountNumber = settingInputs.accountNumber ? settingInputs.accountNumber.value.trim() : '';
        profile.bank.branchName = settingInputs.branchName ? settingInputs.branchName.value.trim() : '';
        profile.bank.branchCode = settingInputs.branchCode ? settingInputs.branchCode.value.trim() : '';
        profile.bank.swiftCode = settingInputs.swiftCode ? settingInputs.swiftCode.value.trim() : '';
        profile.bank.routingNo = settingInputs.routingNo ? settingInputs.routingNo.value.trim() : '';

        if (settingInputs.defaultNote) {
            settings.defaultNote = settingInputs.defaultNote.value;
        }

        if (settingInputs.currency) {
            const selectedOption = settingInputs.currency.options[settingInputs.currency.selectedIndex];
            settings.currency = settingInputs.currency.value;
            settings.currencySymbol = (selectedOption && selectedOption.getAttribute('data-symbol')) || '$';
        }

        data.profile = profile;
        data.settings = settings;
        await InvoiceDB.save(data);
        syncProfileToPreviewsAndDefaults();
        updateDBBadge();

        if (notify) {
            showToast('Profile & bank details saved successfully!', 'success');
        }
    }

    // Auto-save is TURNED OFF for Settings inputs per user request.
    // Inputs ONLY save when the user explicitly clicks "Save Profile & Bank Details".
    const saveSettingsBtn = document.getElementById('save-settings-btn');
    if (saveSettingsBtn) {
        saveSettingsBtn.addEventListener('click', () => {
            saveAllSettingsFromUI(true);
        });
    }

    // Logo Handlers
    function updateLogoDisplay(base64) {
        if (base64) {
            logoPreviewBox.innerHTML = `<img src="${base64}" alt="Logo">`;
            invoiceLogoContainer.innerHTML = `<img src="${base64}" alt="Logo" style="width: 140px; height: 140px; object-fit: contain;">`;
        } else {
            logoPreviewBox.innerHTML = `<span>No Logo</span>`;
            invoiceLogoContainer.innerHTML = `
                <svg id="Layer_1" xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 400 400" width="140" height="140">
                    <path fill="#ea4e25" d="M38.5,294.7l-12.8,3.2C8.8,268,0,234.3,0,200,0,115.3,52.9,42.8,127.4,13.6v29.5c-59.2,27.5-100.3,87.5-100.3,156.9,0,32.2,8.9,63.7,25.9,91l-14.6,3.7ZM200,0c-7.5,0-15,.4-22.4,1.2v27.3c7.4-1,14.9-1.5,22.4-1.5,95.3,0,172.9,77.6,172.9,172.9,0,6.3-.3,12.6-1,18.8l-236.4,59.6v35.2l228.1-57.5s0,0,0,0c-23.4,67.8-87.8,116.6-163.5,116.6-40.1,0-78.9-13.9-109.8-39.4l-31.5,7.9c36.2,36.2,86.2,58.6,141.3,58.6,93.5,0,172.3-64.6,194-151.5,3-12,4.9-24.3,5.6-36.6.2-3.9.4-7.9.4-11.9C400,89.7,310.3,0,200,0ZM169.6,197c0-2.2,1-4.3,2.6-5.9,1.6-1.6,3.7-2.4,6-2.4s4.4.9,6,2.4c1.6,1.6,2.5,3.7,2.6,5.9v50.9l34.2-8.6v-42.3c0-10.8-4.2-19.9-12.6-27.3-8.4-7.4-18.4-11.1-30.1-11.1-2.9,0-5.7-.1-8.5-.4V2.3c-11.6,1.8-23,4.6-34.1,8.4v250.1l34.2-8.6v-55.2ZM295.8,217.5v-7.2c-5.5,4.1-11.8,7.1-18.4,8.9-24.1,6.4-43.6-7.5-43.6-31.1s19.5-47.9,43.6-54.4c6-1.7,12.3-2,18.4-.9v-3.8l34.1-9.1v88.4l-34.2,9.2ZM286.1,174.4c0-1.7-.5-3.4-1.4-4.9-.9-1.5-2.3-2.6-3.9-3.3-1.6-.7-3.4-.9-5.1-.6-1.7.3-3.3,1.1-4.5,2.4-1.2,1.2-2.1,2.8-2.4,4.5-.4,1.7-.2,3.5.5,5.1.7,1.6,1.8,3,3.2,4,1.4,1,3.1,1.5,4.9,1.5,1.1,0,2.3-.2,3.3-.6,1.1-.4,2-1.1,2.8-1.9.8-.8,1.5-1.7,1.9-2.8.4-1,.7-2.2.7-3.3ZM118.3,141.4v-16.6l-34.1,9v16.6l34.1-9ZM84.2,326.7l34.1-8.6v-160l-34.1,8.5v124.8l-26.5,6.7-27.8,7c6.5,10.5,14,20.4,22.4,29.6l5-1.3,25.5-6.4,1.4-.3Z" />
                </svg>
            `;
        }
    }

    logoInput.addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = async (ev) => {
            const base64 = ev.target.result;
            const profile = InvoiceDB.getProfile();
            profile.companyLogo = base64;
            await InvoiceDB.saveProfile(profile);
            updateLogoDisplay(base64);
            showToast('Logo updated successfully!', 'success');
        };
        reader.readAsDataURL(file);
    });

    removeLogoBtn.addEventListener('click', async () => {
        const profile = InvoiceDB.getProfile();
        profile.companyLogo = null;
        await InvoiceDB.saveProfile(profile);
        updateLogoDisplay(null);
        showToast('Logo removed', 'info');
    });

    /* ---------------- JSON Database Backup & Restore ---------------- */

    function updateDBBadge() {
        const dbBadgeText = document.getElementById('db-badge-text');
        const dbClientsCount = document.getElementById('db-clients-count');
        const dbInvoicesCount = document.getElementById('db-invoices-count');
        const dbLastUpdated = document.getElementById('db-last-updated');

        const clients = InvoiceDB.getClients();
        const invoices = InvoiceDB.getInvoices();
        const data = InvoiceDB.getData();

        if (dbBadgeText) dbBadgeText.textContent = `JSON DB: Synced (${invoices.length})`;
        if (dbClientsCount) dbClientsCount.textContent = clients.length;
        if (dbInvoicesCount) dbInvoicesCount.textContent = invoices.length;
        if (dbLastUpdated && data.lastUpdated) {
            const timeStr = new Date(data.lastUpdated).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
            dbLastUpdated.textContent = `Updated ${timeStr}`;
        }
    };

    exportDataBtn.addEventListener('click', () => {
        try {
            const filename = InvoiceDB.exportDatabaseJSON();
            showToast(`JSON database backup downloaded (${filename})`, 'success');
        } catch (e) {
            alert('Failed to export database: ' + e.message);
        }
    });

    if (quickExportJsonBtn) {
        quickExportJsonBtn.addEventListener('click', () => {
            try {
                const filename = InvoiceDB.exportDatabaseJSON();
                showToast(`JSON database backup downloaded (${filename})`, 'success');
            } catch (e) {
                alert('Failed to export database: ' + e.message);
            }
        });
    }

    importInput.addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (!file) return;

        if (!confirm('Importing this JSON database will overwrite current records. Proceed with restore?')) {
            e.target.value = '';
            return;
        }

        const reader = new FileReader();
        reader.onload = async (event) => {
            try {
                const res = await InvoiceDB.importDatabaseJSON(event.target.result);
                alert(`Database restored successfully!\nRestored ${res.clientsCount} clients and ${res.invoicesCount} invoices.`);
                location.reload();
            } catch (err) {
                alert('Restore Failed: ' + err.message);
            }
        };
        reader.readAsText(file);
    });

    resetDataBtn.addEventListener('click', async () => {
        const confirmed = confirm('WARNING: Are you sure you want to delete all data?\n\nThis will permanently delete ALL invoices, clients, profile settings, bank details, and logo from the database.');
        if (confirmed) {
            try {
                await InvoiceDB.deleteAllData();
                showToast('All database records deleted!', 'info');
                setTimeout(() => {
                    location.reload();
                }, 400);
            } catch (err) {
                console.error(err);
                alert('Failed to delete data: ' + err.message);
            }
        }
    });

    /* ---------------- Custom Select Engine ---------------- */

    /**
     * initCustomSelects() — Scans for all <select> elements and wraps them
     * with a fully custom, styled dropdown that matches the dark design language.
     * The original <select> is kept hidden and stays in sync for form reads.
     * @param {Element} [root=document] - Root element to search within.
     */
    function initCustomSelects(root = document) {
        root.querySelectorAll('select:not([data-cs-init])').forEach(sel => {
            sel.setAttribute('data-cs-init', '1');

            // Determine if compact (table row) mode
            const isCompact = sel.classList.contains('table-status-select') ||
                              sel.closest('td') !== null;

            // Build wrapper
            const wrapper = document.createElement('div');
            wrapper.className = 'cs-wrapper' + (isCompact ? ' cs-compact' : '');

            // Insert wrapper before select in DOM, then move select inside
            sel.parentNode.insertBefore(wrapper, sel);
            wrapper.appendChild(sel);

            // Build trigger button
            const trigger = document.createElement('div');
            trigger.className = 'cs-trigger';
            trigger.tabIndex = 0;
            trigger.setAttribute('role', 'combobox');
            trigger.setAttribute('aria-haspopup', 'listbox');
            trigger.setAttribute('aria-expanded', 'false');

            const textEl = document.createElement('span');
            textEl.className = 'cs-trigger-text';

            const arrowEl = document.createElement('span');
            arrowEl.className = 'cs-arrow';
            arrowEl.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 256 256" fill="currentColor"><path d="M213.66,101.66l-80,80a8,8,0,0,1-11.32,0l-80-80A8,8,0,0,1,53.66,90.34L128,164.69l74.34-74.35a8,8,0,0,1,11.32,11.32Z"/></svg>`;

            trigger.appendChild(textEl);
            trigger.appendChild(arrowEl);
            wrapper.appendChild(trigger);

            // Build dropdown panel
            const dropdown = document.createElement('div');
            dropdown.className = 'cs-dropdown';
            dropdown.setAttribute('role', 'listbox');

            const inner = document.createElement('div');
            inner.className = 'cs-dropdown-inner';
            dropdown.appendChild(inner);
            wrapper.appendChild(dropdown);

            // Sync trigger text from select value
            function syncTriggerText() {
                const opt = sel.options[sel.selectedIndex];
                if (opt && opt.value !== '') {
                    textEl.textContent = opt.textContent;
                    textEl.classList.remove('placeholder');
                } else if (opt) {
                    textEl.textContent = opt.textContent;
                    textEl.classList.add('placeholder');
                } else {
                    textEl.textContent = sel.getAttribute('placeholder') || 'Select...';
                    textEl.classList.add('placeholder');
                }
            }

            // Build option items
            function rebuildOptions() {
                inner.innerHTML = '';
                Array.from(sel.options).forEach((opt, idx) => {
                    const item = document.createElement('div');
                    item.className = 'cs-option' + (idx === sel.selectedIndex ? ' selected' : '');
                    item.textContent = opt.textContent;
                    item.dataset.value = opt.value;
                    item.addEventListener('click', (e) => {
                        e.stopPropagation();
                        sel.value = opt.value;
                        sel.dispatchEvent(new Event('change', { bubbles: true }));
                        sel.dispatchEvent(new Event('input',  { bubbles: true }));
                        syncTriggerText();
                        rebuildOptions();
                        closeDropdown();
                    });
                    inner.appendChild(item);
                });
            }

            function openDropdown() {
                // Close any other open custom selects
                document.querySelectorAll('.cs-wrapper.open').forEach(w => {
                    if (w !== wrapper) {
                        w.classList.remove('open');
                        w.querySelector('.cs-trigger')?.setAttribute('aria-expanded', 'false');
                    }
                });
                rebuildOptions();
                wrapper.classList.add('open');
                trigger.setAttribute('aria-expanded', 'true');
            }

            function closeDropdown() {
                wrapper.classList.remove('open');
                trigger.setAttribute('aria-expanded', 'false');
            }

            function toggleDropdown() {
                wrapper.classList.contains('open') ? closeDropdown() : openDropdown();
            }

            // Events
            trigger.addEventListener('click', (e) => { e.stopPropagation(); toggleDropdown(); });
            trigger.addEventListener('keydown', (e) => {
                if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggleDropdown(); }
                if (e.key === 'Escape') closeDropdown();
                if (e.key === 'ArrowDown') {
                    e.preventDefault();
                    if (!wrapper.classList.contains('open')) openDropdown();
                    const items = inner.querySelectorAll('.cs-option');
                    const cur = inner.querySelector('.cs-option.selected');
                    const idx = Array.from(items).indexOf(cur);
                    if (idx < items.length - 1) items[idx + 1].click();
                }
                if (e.key === 'ArrowUp') {
                    e.preventDefault();
                    if (!wrapper.classList.contains('open')) openDropdown();
                    const items = inner.querySelectorAll('.cs-option');
                    const cur = inner.querySelector('.cs-option.selected');
                    const idx = Array.from(items).indexOf(cur);
                    if (idx > 0) items[idx - 1].click();
                }
            });

            // Store sync method on select element
            sel._syncCustomSelect = () => {
                syncTriggerText();
                rebuildOptions();
            };

            // Keep custom select in sync if native select changes externally
            sel.addEventListener('change', () => { syncTriggerText(); rebuildOptions(); });
            sel.addEventListener('input',  () => { syncTriggerText(); rebuildOptions(); });

            // Initial render
            syncTriggerText();
        });

        // Global close on outside click
        document.addEventListener('click', () => {
            document.querySelectorAll('.cs-wrapper.open').forEach(w => {
                w.classList.remove('open');
                w.querySelector('.cs-trigger')?.setAttribute('aria-expanded', 'false');
            });
        }, { capture: false });
    }

    /**
     * Synchronizes a custom select wrapper with its underlying <select> value.
     * Useful when the select's .value is updated programmatically.
     * @param {HTMLSelectElement} sel
     */
    function syncCustomSelect(sel) {
        if (!sel) return;
        if (typeof sel._syncCustomSelect === 'function') {
            sel._syncCustomSelect();
        } else {
            sel.dispatchEvent(new Event('change', { bubbles: true }));
        }
    }
    window.syncCustomSelect = syncCustomSelect;

    /* ---------------- App Initialization ---------------- */

    // 1. Load settings & defaults
    loadProfileAndSettings();

    // 2. Initialize custom select dropdowns (replaces all native <select> elements)
    initCustomSelects();

    // 3. Populate clients dropdown (without selecting any client by default)
    populateClientDropdown();
    initCustomSelects(); // re-init after client options are built

    // 3. Always open a fresh new invoice on startup/refresh
    startNewInvoice(true);

    // 4. Update earnings display & DB badge
    updateEarningsDisplay();
    updateDBBadge();

    // 5. Responsive auto-fit zoom for small screens
    autoFitZoom();
    window.addEventListener('resize', autoFitZoom);
    window.addEventListener('orientationchange', () => setTimeout(autoFitZoom, 150));
});
