const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const http = require('http');

const PORT = 18789;
let win = null;
let failCount = 0;

function createWindow() {
  win = new BrowserWindow({
    width: 230, height: 130, x: 0, y: 0,
    frame: false, transparent: true, alwaysOnTop: true,
    skipTaskbar: true, resizable: false, hasShadow: false, show: false,
    webPreferences: { nodeIntegration: true, contextIsolation: false },
  });

  win.loadFile(path.join(__dirname, 'index.html'));

  win.once('ready-to-show', () => {
    const { screen } = require('electron');
    const { width: sw } = screen.getPrimaryDisplay().workAreaSize;
    win.setPosition(sw - 500, 200);
    win.show();
  });

  ipcMain.on('set-ignore-mouse', (event, ignore) => {
    if (win) win.setIgnoreMouseEvents(ignore, { forward: true });
  });
  ipcMain.on('close-app', () => app.quit());

  // Poll data; if server disconnects for ~12s, exit
  function poll() {
    let timedOut = false;
    const req = http.get(`http://127.0.0.1:${PORT}`, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        if (win && !win.isDestroyed()) {
          try {
            win.webContents.send('hwmon-data', JSON.parse(data));
            failCount = 0;
          } catch(e) {}
        }
      });
    });
    req.on('error', () => {
      if (!timedOut) {
        failCount++;
        if (failCount > 12) app.quit();
      }
    });
    req.on('timeout', () => {
      timedOut = true;
      req.destroy();
      failCount++;
      if (failCount > 12) app.quit();
    });
    req.setTimeout(2000);
  }

  setTimeout(() => { poll(); setInterval(poll, 500); }, 1000);
}

app.whenReady().then(createWindow);
app.on('window-all-closed', () => app.quit());
