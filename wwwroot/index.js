
var importObject = {
  imports: { imported_func: arg => console.log(arg) }
};


var app = new Vue(
  {
  // The div, to which our app is rendered to:
  el: '#app',
  // the context of the app
  data: {
    message: 'Hello, ',
    username: 'Dude',
    items: [],
    ValErrorTo: 'A',
    ValErrorFrom: 'B',
    hasClocktoError: false,
    hasClockfromError: false,
    hasComment: false,
    Clockfrom: '',
    Clockto: '',
    Comment:''    
  },

  // any clientside code of our app.
  methods: {  },
  mounted:function()
  {
    WebAssembly.instantiateStreaming(fetch('test.wasm'), importObject)
               .then(obj => {
                 console.log( "WASM retuned: " + obj.instance.exports.hello_world(1011))
               });
  },
})