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
})

async function run() {
  const response = fetch("./wasm/deno.wasm");
  // if there is the need to have to wasm buffer
  // const buffer = await response.arrayBuffer();
  // const wasmModule = await WebAssembly.compile(response);

  // compileStreaming extract directly the response
  // using this make it even possible to not await the response itself while fetching
  // const wasmModule = await WebAssembly.compileStreaming(response);
  // const wasmInstance = await WebAssembly.instantiate(wasmModule);
  // const {
  //     square,
  // } = wasmInstance.exports;

  // to do all thw work above we can use instantiateStreaming
  const {module, instance} = await WebAssembly.instantiateStreaming(response)
  // module can be instantiated multiple times
  const other_instance = await WebAssembly.instantiate(module)

  // not sure if this is a clean way to return the promise
  return instance.exports
}
let instance = run()

function square()
{
  instance.then(obj => {
    console.log( "square: " + obj.square(3))
  });
}

document.getElementById("ok_button").onclick = function() { square() }


// websocket implementation
let socket = new WebSocket("ws://127.0.0.1:3013");

socket.onopen = function()
{
    socket.send("thefux says hello")
}

socket.onmessage = function(e)
{
    console.log("message: ", e.data);
}
