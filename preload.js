const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
    exportPdf: (params) => ipcRenderer.invoke('export-pdf', params),
    selectDirectory: () => ipcRenderer.invoke('select-directory'),
    exportPdfBuffer: (params) => ipcRenderer.invoke('export-pdf-buffer', params),
    saveFileToPath: (params) => ipcRenderer.invoke('save-file-to-path', params)
});

