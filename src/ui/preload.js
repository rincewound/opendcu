let {
	webFrame
} = require("electron");

var _process = process;
process.once('loaded', function() {
  global.process= _process;
});
