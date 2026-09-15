import init, { render_preview } from "./pkg/aozora_rs_wasm.js";

let wasmReady = false;

async function loadWasm() {
  await init();
  wasmReady = true;
  postMessage({ type: "READY" });
}

onmessage = async (e) => {
  if (e.data.type === "PARSE") {
    if (!wasmReady) return;
    const decoder = new TextDecoder();
    const text = decoder.decode(new Uint8Array(e.data.buffer));

    try {
      const html = render_preview(text);
      postMessage({ type: "RESULT", html });
    } catch (err) {
      postMessage({
        type: "RESULT",
        html: `<p style="color:#e06060;font-family:Inter,sans-serif;font-size:0.9rem;padding:1em;">${err.message || String(err)}</p>`,
      });
    }
  }
};

loadWasm();
