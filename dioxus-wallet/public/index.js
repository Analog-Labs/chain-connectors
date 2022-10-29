import init from "./assets/dioxus/dioxus.js";
init("./assets/dioxus/dioxus_bg.wasm").then(wasm => {
  if (wasm.__wbindgen_start == undefined) {
    wasm.main();
  }
});
