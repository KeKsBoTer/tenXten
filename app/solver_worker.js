importScripts('./solver/tenxten.js');

const init = wasm_bindgen("./solver/tenxten_bg.wasm").catch(err => {
    setTimeout(() => {
        throw err;
    });
    throw err;
});

self.onmessage = async ({ data }) => {
    await init;
    let result = wasm_bindgen.solve(data[0], data[1]);
    self.postMessage(result || null);
};