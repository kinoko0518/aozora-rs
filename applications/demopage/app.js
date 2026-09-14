import init, {
  build_epub,
  render_standalone_html_from_bytes,
} from "./pkg/aozora_rs_wasm.js";

// web worker 準備
const worker = new Worker("web_worker.js", { type: "module" });

worker.onmessage = (e) => {
  if (e.data.type === "READY") {
    console.log("Wasm Worker is ready!");
    state.wasmReady = true;
    updatePreview();
  } else if (e.data.type === "RESULT") {
    previewArea.innerHTML = e.data.html;
  }
};

// === 状態管理 ===
const state = {
  wasmReady: false,
  fileBytes: null,
  fileName: null,
  fileType: null,
  isVertical: true,
  useMiyabi: true,
  usePrelude: true,
  considerGaiji: true,
};

// === DOM要素 ===
const $ = (id) => document.getElementById(id);
const textarea = $("editor-textarea");
const previewArea = $("preview-area-container");
const encodingSwitch = $("encoding-switch");
const labelUtf8 = $("label-utf8");
const labelSjis = $("label-sjis");
const directionSwitch = $("direction-switch");
const labelHorizontal = $("label-horizontal");
const labelVertical = $("label-vertical");
const cbMiyabi = $("cb-miyabi");
const cbPrelude = $("cb-prelude");
const cbGaiji = $("cb-gaiji");
const fileInput = $("file-input");
const fileInfo = $("file-info");
const converterActions = $("converter-actions");
const btnDownload = $("btn-download");
const btnXhtml = $("btn-xhtml");
const spinnerDownload = $("spinner-download");
const spinnerXhtml = $("spinner-xhtml");
const statusBar = $("status-bar");

// === ユーティリティ ===
function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

function getEncoding() {
  return encodingSwitch.checked ? "shift_jis" : "utf-8";
}

function setStatus(msg, type = "info") {
  statusBar.textContent = msg;
  statusBar.className = `status-bar status-bar--${type}`;
}

function clearStatus() {
  statusBar.textContent = "";
  statusBar.className = "status-bar";
}

// === リアルタイムプレビュー ===
function updatePreview() {
  if (!state.wasmReady) return;

  const text = textarea.value;
  const encoder = new TextEncoder();
  const buffer = encoder.encode(text).buffer;
  worker.postMessage({ type: "PARSE", buffer: buffer }, [buffer]);
}

const debouncedPreview = debounce(updatePreview, 50);

// === エンコーディング切替 ===
function updateEncodingLabels() {
  const isShiftJIS = encodingSwitch.checked;
  labelUtf8.classList.toggle("toggle-group__label--active", !isShiftJIS);
  labelSjis.classList.toggle("toggle-group__label--active", isShiftJIS);
}

// === 書字方向切替 ===
function updateDirectionLabels() {
  const isVertical = directionSwitch.checked;
  labelHorizontal.classList.toggle("toggle-group__label--active", !isVertical);
  labelVertical.classList.toggle("toggle-group__label--active", isVertical);
  state.isVertical = isVertical;
}

// === ファイル読み込み ===
async function handleFileUpload(file) {
  if (!file) return;

  const ext = file.name.split(".").pop().toLowerCase();
  if (ext !== "zip" && ext !== "txt") {
    setStatus(".zip または .txt ファイルを選択してください。", "error");
    return;
  }

  state.fileName = file.name;
  state.fileType = ext;
  state.fileBytes = new Uint8Array(await file.arrayBuffer());

  fileInfo.textContent = file.name;
  converterActions.classList.add("converter__actions--visible");
  clearStatus();
}

// === EPUBダウンロード ===
async function handleDownload() {
  if (!state.fileBytes) return;

  spinnerDownload.classList.add("spinner--active");
  btnDownload.disabled = true;
  setStatus("EPUB を生成中…", "info");

  try {
    const options = {
      encoding: getEncoding(),
      isVertical: state.isVertical,
      useMiyabi: state.useMiyabi,
      usePrelude: state.usePrelude,
      considerGaiji: state.considerGaiji,
    };

    const epubBytes = build_epub(state.fileBytes, options);

    if (!epubBytes || epubBytes.length === 0) {
      throw new Error("EPUBファイルの生成に失敗しました。");
    }

    const blob = new Blob([epubBytes], { type: "application/epub+zip" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    const baseName = state.fileName.replace(/\.[^.]+$/, "");
    a.download = `${baseName}.epub`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    setStatus("EPUB のダウンロードを開始しました。", "success");
  } catch (e) {
    setStatus(`エラー: ${e.message || String(e)}`, "error");
  } finally {
    spinnerDownload.classList.remove("spinner--active");
    btnDownload.disabled = false;
  }
}

// === HTMLで読む ===
async function handleXhtmlView() {
  if (!state.fileBytes) return;

  spinnerXhtml.classList.add("spinner--active");
  btnXhtml.disabled = true;
  setStatus("XHTML を生成中…", "info");

  try {
    const options = {
      encoding: getEncoding(),
      isVertical: state.isVertical,
      useMiyabi: state.useMiyabi,
      usePrelude: state.usePrelude,
      considerGaiji: state.considerGaiji,
    };

    const html = render_standalone_html_from_bytes(state.fileBytes, options);

    const blob = new Blob([html], { type: "text/html; charset=utf-8" });
    const url = URL.createObjectURL(blob);
    window.open(url, "_blank");

    setStatus("XHTML を新しいタブで開きました。", "success");
  } catch (e) {
    setStatus(`エラー: ${e.message || String(e)}`, "error");
  } finally {
    spinnerXhtml.classList.remove("spinner--active");
    btnXhtml.disabled = false;
  }
}

// === 初期化 ===
async function main() {
  try {
    await init();
    state.wasmReady = true;

    textarea.addEventListener("input", debouncedPreview);
    encodingSwitch.addEventListener("change", updateEncodingLabels);
    directionSwitch.addEventListener("change", updateDirectionLabels);
    cbMiyabi.addEventListener("change", () => {
      state.useMiyabi = cbMiyabi.checked;
    });
    cbPrelude.addEventListener("change", () => {
      state.usePrelude = cbPrelude.checked;
    });
    cbGaiji.addEventListener("change", () => {
      state.considerGaiji = cbGaiji.checked;
    });
    fileInput.addEventListener("change", (e) =>
      handleFileUpload(e.target.files[0]),
    );
    btnDownload.addEventListener("click", handleDownload);
    btnXhtml.addEventListener("click", handleXhtmlView);

    updateEncodingLabels();
    updateDirectionLabels();
    updatePreview();
  } catch (e) {
    console.error("初期化に失敗しました:", e);
    setStatus(`初期化エラー: ${e.message || String(e)}`, "error");
  }
}

main();
