importScripts('./solver/tenxten.js');

const init = wasm_bindgen("./solver/tenxten_bg.wasm").catch(err => {
    setTimeout(() => {
        throw err;
    });
    throw err;
});

self.onmessage = async event => {
    await init;
    let result = wasm_bindgen.solve(event.data);
    self.postMessage(result || null);
};