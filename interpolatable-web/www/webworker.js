var module = import("../pkg/interpolatable_web.js");

async function init() {
  console.log("Loading the module");
  let wasm = await module;
  console.log("Loaded");
  self.postMessage({ ready: true });
  self.onmessage = async (event) => {
    // make sure loading is done
    const { font } = event.data;
    try {
      const results = JSON.parse(wasm.check_font(font));
      self.postMessage({ results: results });
    } catch (error) {
      self.postMessage({ error: error.message });
    }
  };
}
init();
