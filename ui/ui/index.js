// const { app, BrowserWindow } = require('electron');
// const path = require('path');
// 
// // Handle creating/removing shortcuts on Windows when installing/uninstalling.
// if (require('electron-squirrel-startup')) { // eslint-disable-line global-require
//   app.quit();
// }
// 
// const createWindow = () => {
//   // Create the browser window.
//   const mainWindow = new BrowserWindow({
//     width: 800,
//     height: 600,
//   });
// 
//   // and load the index.html of the app.
//   mainWindow.loadFile(path.join(__dirname, './index.html'));
// 
//   // Open the DevTools.
//   mainWindow.webContents.openDevTools();
// };
// 
// // This method will be called when Electron has finished
// // initialization and is ready to create browser windows.
// // Some APIs can only be used after this event occurs.
// app.on('ready', createWindow);
// 
// // Quit when all windows are closed.
// app.on('window-all-closed', () => {
//   // On OS X it is common for applications and their menu bar
//   // to stay active until the user quits explicitly with Cmd + Q
//   if (process.platform !== 'darwin') {
//     app.quit();
//   }
// });
// 
// app.on('activate', () => {
//   // On OS X it's common to re-create a window in the app when the
//   // dock icon is clicked and there are no other windows open.
//   if (BrowserWindow.getAllWindows().length === 0) {
//     createWindow();
//   }
// });

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.

let {
	app,
	protocol,
	BrowserWindow
} = require("electron");
let {
	readFile
} = require("fs");
let {
	extname
} = require("path");
let {
	URL
} = require("url");

let createProtocol = (scheme, normalize = true) => {
	protocol.registerBufferProtocol(scheme,
		(request, respond) => {
			let pathName = new URL(request.url).pathname;

			// Needed in case URL contains spaces
			pathName = decodeURI(pathName);

			readFile(__dirname + "/" + pathName, (error, data) => {
				let extension = extname(pathName).toLowerCase();
				let mimeType = "";
				if (extension === ".js") {
					mimeType = "text/javascript";
				} else if (extension === ".html") {
					mimeType = "text/html";
				} else if (extension === ".css") {
					mimeType = "text/css";
				} else if (extension === ".svg" || extension ===
					".svgz") {
					mimeType = "image/svg+xml";
				} else if (extension === ".json") {
					mimeType = "application/json";
				}
				respond({
					mimeType,
					data
				});
			});
		},
		(error) => {
			if (error) {
				console.error(`Failed to register ${scheme} protocol`,
					error);
			}
		}
	);
}

// Standard scheme must be registered before the app is ready
// https://gist.github.com/dbkr/e898624be6d53590ebf494521d868fec
protocol.registerSchemesAsPrivileged([{
    scheme: 'app',
    privileges: { standard: true, secure: true, supportFetchAPI: true },
}]);


app.on("ready", () => {
	createProtocol("app");
	let browserWindow = new BrowserWindow({
		webPreferences: {
			preload: `${__dirname}/preload.js`,
			nodeIntegration: false,
			contextIsolation: true
		}
	});
	//browserWindow.webContents.openDevTools();
	browserWindow.loadFile("index.html");
});