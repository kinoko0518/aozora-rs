import init, {
  generate_embedding_xhtml,
  parse_to_book_data,
  build_epub_bytes,
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

// === miyabi CSS（HTMLビュー用） ===
let miyabiCssText = null;

async function loadMiyabiCss() {
  const url = new URL("../ayame/assets/miyabi.css", import.meta.url).href;
  const resp = await fetch(url);
  miyabiCssText = await resp.text();
}

// === prelude CSS（HTMLビュー用） ===
let preludeCssText = null;

async function loadPreludeCss() {
  const url = new URL(
    "../../aozora-rs/aozora-rs/css/prelude.css",
    import.meta.url,
  ).href;
  const resp = await fetch(url);
  preludeCssText = await resp.text();
}

// === Preview CSS（インライン化用） ===
let previewCssText = null;

async function loadPreviewCss() {
  const url = new URL("./preview.css", import.meta.url).href;
  const resp = await fetch(url);
  previewCssText = await resp.text();
}

// === ユーティリティ ===
function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

function getEncoding() {
  return encodingSwitch.checked ? "sjis" : "utf8";
}

function setStatus(msg, type = "info") {
  statusBar.textContent = msg;
  statusBar.className = `status-bar status-bar--${type}`;
}

function clearStatus() {
  statusBar.textContent = "";
  statusBar.className = "status-bar";
}

// === XHTMLビューア用の完全なHTML ===
function buildFullXhtml(xhtmlBodies) {
  const cssParts = [];

  if (state.usePrelude && preludeCssText) {
    cssParts.push(preludeCssText);
  }
  if (state.useMiyabi && miyabiCssText) {
    cssParts.push(miyabiCssText);
  }

  const writingMode = state.isVertical ? "vertical-rl" : "horizontal-tb";
  cssParts.push(`body {
    writing-mode: ${writingMode};
    -webkit-writing-mode: ${writingMode};
    margin: 0;
    padding: 0;
    color: #1a1a1a;
    background: #faf8f0;
  }`);

  const combinedCss = cssParts.join("\n");
  const separator = state.isVertical
    ? ""
    : '<hr style="margin:2em 0;border:none;border-top:1px solid #ccc;">';

  return `<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>XHTML プレビュー</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Shippori+Mincho:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>
${combinedCss}
</style>
</head>
<body>${xhtmlBodies.join(separator)}</body>
</html>`;
}

// === リアルタイムプレビュー ===
function updatePreview() {
  if (!state.wasmReady || !previewCssText) return;

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

// === .txtを.zipにラップ ===
async function wrapTxtAsZip(txtBytes) {
  const zip = new JSZip();
  zip.file("input.txt", txtBytes);
  return await zip.generateAsync({ type: "uint8array" });
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
    let zipBytes;
    if (state.fileType === "txt") {
      zipBytes = await wrapTxtAsZip(state.fileBytes);
    } else {
      zipBytes = state.fileBytes;
    }

    const encoding = getEncoding();
    const epubBytes = build_epub_bytes(
      zipBytes,
      encoding,
      state.isVertical,
      state.useMiyabi,
      state.usePrelude,
      state.considerGaiji,
    );

    if (epubBytes.length === 0) {
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
    setStatus(`エラー: ${e.message}`, "error");
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
    let textContent;

    if (state.fileType === "txt") {
      const encoding = getEncoding();
      if (encoding === "sjis") {
        const decoder = new TextDecoder("shift_jis");
        textContent = decoder.decode(state.fileBytes);
      } else {
        const decoder = new TextDecoder("utf-8");
        textContent = decoder.decode(state.fileBytes);
      }
    } else {
      const zip = await JSZip.loadAsync(state.fileBytes);
      let txtFile = null;
      zip.forEach((path, entry) => {
        if (!entry.dir && path.endsWith(".txt")) {
          txtFile = entry;
        }
      });

      if (!txtFile) {
        throw new Error("zip内にtxtファイルが見つかりませんでした。");
      }

      const encoding = getEncoding();
      if (encoding === "sjis") {
        const bytes = await txtFile.async("uint8array");
        const decoder = new TextDecoder("shift_jis");
        textContent = decoder.decode(bytes);
      } else {
        textContent = await txtFile.async("string");
      }
    }

    const bookData = parse_to_book_data(textContent);

    if (!bookData.xhtmls || bookData.xhtmls.length === 0) {
      const err = bookData.errors;
      bookData.free();
      throw new Error(`XHTMLの生成に失敗しました: ${err}`);
    }

    const html = buildFullXhtml(bookData.xhtmls);
    bookData.free();

    const blob = new Blob([html], { type: "text/html; charset=utf-8" });
    const url = URL.createObjectURL(blob);
    window.open(url, "_blank");

    setStatus("XHTML を新しいタブで開きました。", "success");
  } catch (e) {
    setStatus(`エラー: ${e.message}`, "error");
  } finally {
    spinnerXhtml.classList.remove("spinner--active");
    btnXhtml.disabled = false;
  }
}

// === 初期化 ===
async function main() {
  try {
    await Promise.all([
      init(),
      loadPreviewCss(),
      loadMiyabiCss(),
      loadPreludeCss(),
    ]);
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
    setStatus(`初期化エラー: ${e.message}`, "error");
  }
}

main();
