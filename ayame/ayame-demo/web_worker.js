import init, {
    generate_standalone_xhtml,
} from './pkg/aozora_rs_wasm.js';

let wasmReady = false;

async function loadWasm() {
    await init();
    wasmReady = true;
    postMessage({ type: 'READY' });
}

onmessage = async (e) => {
    if (e.data.type === 'PARSE') {
        if (!wasmReady) return;
        const decoder = new TextDecoder();
        const text = decoder.decode(new Uint8Array(e.data.buffer));

        const result = generate_standalone_xhtml(text, '');

        if (!result.result && result.occured_error) {
            postMessage({ type:"RESULT", html: `<p style="color:#e06060;font-family:Inter,sans-serif;font-size:0.9rem;padding:1em;">${escapeHtml(result.occured_error)}</p>` });
        } else {
            postMessage({ type:"RESULT", html: result.result });
        }
        
        result.free();
    }
}

function escapeHtml(str) {
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

loadWasm();
