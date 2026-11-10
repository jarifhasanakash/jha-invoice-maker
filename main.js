const { app, BrowserWindow, ipcMain, dialog } = require('electron');
const path = require('path');
const fs = require('fs');
const crypto = require('crypto');

let mainWindow;

function createWindow() {
    mainWindow = new BrowserWindow({
        width: 1380,
        height: 920,
        minWidth: 800,
        minHeight: 600,
        title: 'Invoice Maker',
        icon: path.join(__dirname, 'jha-icon.png'),
        backgroundColor: '#141416',
        show: false,
        autoHideMenuBar: true,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            nodeIntegration: false,
            spellcheck: false
        }
    });

    mainWindow.loadFile(path.join(__dirname, 'index.html'));

    mainWindow.once('ready-to-show', () => {
        mainWindow.show();
    });
}

app.whenReady().then(() => {
    createWindow();

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
});

// Helper to optionally encrypt PDF with owner password (AES-256 permissions lock)
async function maybeEncryptPdf(pdfBuffer, password) {
    if (!password || typeof password !== 'string' || password.trim().length === 0) {
        return pdfBuffer;
    }
    try {
        const { encryptPDF } = require('./pdf-crypto.js');
        const pwd = password.trim();

        // userPassword is empty string ('') so anyone can open, view, and print freely without being prompted for a password.
        // Content copying and accessibility extraction are allowed without a password.
        // ownerPassword is set to pwd so editing, modifying, annotating, form filling, or document assembly requires entering the password in PDF editors.
        const encryptedBytes = await encryptPDF(new Uint8Array(pdfBuffer), '', {
            ownerPassword: pwd,
            algorithm: 'AES-256',
            allowPrinting: true,
            allowHighQualityPrint: true,
            allowCopying: true,
            allowExtraction: true,
            allowModifying: false,
            allowAnnotating: false,
            allowFillingForms: false,
            allowAssembly: false
        });
        return Buffer.from(encryptedBytes);
    } catch (encErr) {
        console.error('PDF Encryption error:', encErr);
        throw new Error(`PDF encryption failed: ${encErr.message || encErr}`);
    }
}

// Native Single-Page Vector PDF Export Handler
ipcMain.handle('export-pdf', async (event, args) => {
    try {
        const width = args.width || 1080;
        const height = args.height || 1122;
        const filename = args.filename || 'Invoice - 0001.pdf';

        // Convert CSS pixels (96 DPI) to microns (1 in = 96 px = 25400 microns)
        const widthMicrons = Math.round((width / 96) * 25400);
        const heightMicrons = Math.round((height / 96) * 25400);

        const pdfData = await mainWindow.webContents.printToPDF({
            printBackground: true,
            margins: { marginType: 'none' },
            pageSize: {
                width: widthMicrons,
                height: heightMicrons
            },
            preferCSSPageSize: true
        });

        const finalBuffer = await maybeEncryptPdf(pdfData, args.password);

        // Prompt native Windows Save File Dialog
        const { canceled, filePath } = await dialog.showSaveDialog(mainWindow, {
            title: 'Save Invoice PDF',
            defaultPath: path.join(app.getPath('downloads'), filename),
            filters: [
                { name: 'PDF Documents', extensions: ['pdf'] }
            ]
        });

        if (canceled || !filePath) {
            return { canceled: true };
        }

        fs.writeFileSync(filePath, finalBuffer);
        return { success: true, filePath };
    } catch (err) {
        console.error('Error generating desktop PDF:', err);
        return { error: err.message };
    }
});

// Native Select Directory Handler for Bulk Exports
ipcMain.handle('select-directory', async () => {
    try {
        const { canceled, filePaths } = await dialog.showOpenDialog(mainWindow, {
            title: 'Select Destination Folder for Invoices',
            properties: ['openDirectory', 'createDirectory']
        });
        if (canceled || !filePaths || filePaths.length === 0) {
            return { canceled: true };
        }
        return { success: true, folderPath: filePaths[0] };
    } catch (err) {
        return { error: err.message };
    }
});

// Native Vector PDF Buffer Handler
ipcMain.handle('export-pdf-buffer', async (event, args) => {
    try {
        const width = args.width || 1080;
        const height = args.height || 1122;
        const widthMicrons = Math.round((width / 96) * 25400);
        const heightMicrons = Math.round((height / 96) * 25400);

        const pdfData = await mainWindow.webContents.printToPDF({
            printBackground: true,
            margins: { marginType: 'none' },
            pageSize: {
                width: widthMicrons,
                height: heightMicrons
            },
            preferCSSPageSize: true
        });
        const finalBuffer = await maybeEncryptPdf(pdfData, args.password);
        return { success: true, data: Array.from(finalBuffer) };
    } catch (err) {
        return { error: err.message };
    }
});

// Native Save File Directly To Disk
ipcMain.handle('save-file-to-path', async (event, { folderPath, filename, buffer }) => {
    try {
        const target = path.join(folderPath, filename);
        fs.writeFileSync(target, Buffer.from(buffer));
        return { success: true, filePath: target };
    } catch (err) {
        return { error: err.message };
    }
});
