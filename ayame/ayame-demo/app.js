import init, {
    generate_standalone_xhtml,
    parse_to_book_data,
    build_epub_bytes,
} from './pkg/aozora_rs_wasm.js';

// === 状態管理 ===
const state = {
    wasmReady: false,
    fileBytes: null,
    fileName: null,
    fileType: null,
};

// === DOM要素 ===
const $ = (id) => document.getElementById(id);
const textarea = $('editor-textarea');
const previewFrame = $('preview-frame');
const encodingSwitch = $('encoding-switch');
const labelUtf8 = $('label-utf8');
const labelSjis = $('label-sjis');
const fileInput = $('file-input');
const fileInfo = $('file-info');
const converterActions = $('converter-actions');
const btnDownload = $('btn-download');
const btnXhtml = $('btn-xhtml');
const spinnerDownload = $('spinner-download');
const spinnerXhtml = $('spinner-xhtml');
const statusBar = $('status-bar');

// === Preview CSS（インライン化用） ===
let previewCssText = null;

async function loadPreviewCss() {
    const url = new URL('./preview.css', import.meta.url).href;
    const resp = await fetch(url);
    previewCssText = await resp.text();
}

// === EPUB CSS ===
let epubCssTexts = [];

async function loadEpubCss() {
    const urls = [
        new URL('../ayame-core/assets/vertical.css', import.meta.url).href,
        new URL('../ayame-core/assets/prelude.css', import.meta.url).href,
        new URL('../ayame-core/assets/miyabi.css', import.meta.url).href
    ];
    const resps = await Promise.all(urls.map(url => fetch(url)));
    epubCssTexts = await Promise.all(resps.map(r => r.text()));
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
    return encodingSwitch.checked ? 'sjis' : 'utf8';
}

function setStatus(msg, type = 'info') {
    statusBar.textContent = msg;
    statusBar.className = `status-bar status-bar--${type}`;
}

function clearStatus() {
    statusBar.textContent = '';
    statusBar.className = 'status-bar';
}

// === プレビューのXHTML構成 ===
function buildPreviewHtml(xhtmlBody) {
    return `<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Shippori+Mincho:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>${previewCssText || ''}</style>
</head>
<body>${xhtmlBody}</body>
</html>`;
}

// === XHTMLビューア用の完全なHTML ===
function buildFullXhtml(xhtmlBodies) {
    const combinedCss = previewCssText || '';
    const bodyStyle = `
        color: #1a1a1a;
        background: #faf8f0;
    `;
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
body { ${bodyStyle} }
</style>
</head>
<body>${xhtmlBodies.join('<hr style="margin:2em 0;border:none;border-top:1px solid #ccc;">')}</body>
</html>`;
}

// === リアルタイムプレビュー ===
function updatePreview() {
    if (!state.wasmReady || !previewCssText) return;

    const text = textarea.value;
    try {
        const result = generate_standalone_xhtml(text, '');

        if (!result.result && result.occured_error) {
            previewFrame.srcdoc = buildPreviewHtml(
                `<p style="color:#e06060;font-family:Inter,sans-serif;font-size:0.9rem;padding:1em;">${escapeHtml(result.occured_error)}</p>`
            );
        } else {
            const html = buildPreviewHtml(result.result);
            previewFrame.srcdoc = html;
        }

        result.free();
    } catch (e) {
        previewFrame.srcdoc = buildPreviewHtml(
            `<p style="color:#e06060;font-family:Inter,sans-serif;font-size:0.9rem;padding:1em;">${escapeHtml(e.message)}</p>`
        );
    }
}

function escapeHtml(str) {
    return str
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

const debouncedPreview = debounce(updatePreview, 200);

// === エンコーディング切替 ===
function updateEncodingLabels() {
    const isShiftJIS = encodingSwitch.checked;
    labelUtf8.classList.toggle('toggle-group__label--active', !isShiftJIS);
    labelSjis.classList.toggle('toggle-group__label--active', isShiftJIS);
}

// === .txtを.zipにラップ ===
async function wrapTxtAsZip(txtBytes) {
    const zip = new JSZip();
    zip.file('input.txt', txtBytes);
    return await zip.generateAsync({ type: 'uint8array' });
}

// === ファイル読み込み ===
async function handleFileUpload(file) {
    if (!file) return;

    const ext = file.name.split('.').pop().toLowerCase();
    if (ext !== 'zip' && ext !== 'txt') {
        setStatus('.zip または .txt ファイルを選択してください。', 'error');
        return;
    }

    state.fileName = file.name;
    state.fileType = ext;
    state.fileBytes = new Uint8Array(await file.arrayBuffer());

    fileInfo.textContent = file.name;
    converterActions.classList.add('converter__actions--visible');
    clearStatus();
}

// === EPUBダウンロード ===
async function handleDownload() {
    if (!state.fileBytes) return;

    spinnerDownload.classList.add('spinner--active');
    btnDownload.disabled = true;
    setStatus('EPUB を生成中…', 'info');

    try {
        let zipBytes;
        if (state.fileType === 'txt') {
            zipBytes = await wrapTxtAsZip(state.fileBytes);
        } else {
            zipBytes = state.fileBytes;
        }

        const encoding = getEncoding();
        const epubBytes = build_epub_bytes(zipBytes, epubCssTexts, encoding);

        if (epubBytes.length === 0) {
            throw new Error('EPUBファイルの生成に失敗しました。');
        }

        const blob = new Blob([epubBytes], { type: 'application/epub+zip' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        const baseName = state.fileName.replace(/\.[^.]+$/, '');
        a.download = `${baseName}.epub`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        setStatus('EPUB のダウンロードを開始しました。', 'success');
    } catch (e) {
        setStatus(`エラー: ${e.message}`, 'error');
    } finally {
        spinnerDownload.classList.remove('spinner--active');
        btnDownload.disabled = false;
    }
}

// === HTMLで読む ===
async function handleXhtmlView() {
    if (!state.fileBytes) return;

    spinnerXhtml.classList.add('spinner--active');
    btnXhtml.disabled = true;
    setStatus('XHTML を生成中…', 'info');

    try {
        let textContent;

        if (state.fileType === 'txt') {
            const encoding = getEncoding();
            if (encoding === 'sjis') {
                const decoder = new TextDecoder('shift_jis');
                textContent = decoder.decode(state.fileBytes);
            } else {
                const decoder = new TextDecoder('utf-8');
                textContent = decoder.decode(state.fileBytes);
            }
        } else {
            // .zipの場合: JSZipで中のtxtを読む
            const zip = await JSZip.loadAsync(state.fileBytes);
            let txtFile = null;
            zip.forEach((path, entry) => {
                if (!entry.dir && path.endsWith('.txt')) {
                    txtFile = entry;
                }
            });

            if (!txtFile) {
                throw new Error('zip内にtxtファイルが見つかりませんでした。');
            }

            const encoding = getEncoding();
            if (encoding === 'sjis') {
                const bytes = await txtFile.async('uint8array');
                const decoder = new TextDecoder('shift_jis');
                textContent = decoder.decode(bytes);
            } else {
                textContent = await txtFile.async('string');
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

        const blob = new Blob([html], { type: 'text/html; charset=utf-8' });
        const url = URL.createObjectURL(blob);
        window.open(url, '_blank');

        setStatus('XHTML を新しいタブで開きました。', 'success');
    } catch (e) {
        setStatus(`エラー: ${e.message}`, 'error');
    } finally {
        spinnerXhtml.classList.remove('spinner--active');
        btnXhtml.disabled = false;
    }
}

// === 初期化 ===
async function main() {
    try {
        await Promise.all([
            init(),
            loadPreviewCss(),
            loadEpubCss(),
        ]);
        state.wasmReady = true;

        textarea.addEventListener('input', debouncedPreview);
        encodingSwitch.addEventListener('change', updateEncodingLabels);
        fileInput.addEventListener('change', (e) => handleFileUpload(e.target.files[0]));
        btnDownload.addEventListener('click', handleDownload);
        btnXhtml.addEventListener('click', handleXhtmlView);

        updateEncodingLabels();
        updatePreview();
    } catch (e) {
        console.error('初期化に失敗しました:', e);
        setStatus(`初期化エラー: ${e.message}`, 'error');
    }
}

main();
