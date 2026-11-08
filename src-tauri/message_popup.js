(function() {
    'use strict';

    const POPUP_ID = 'wl-message-popup';
    const SHOW_MS = 5000;

    if (window.__wlPopup) window.__wlPopup.destroy();

    const CSS = `
        #${POPUP_ID} {
            position: fixed;
            top: 8px;
            left: 50%;
            transform: translate(-50%, -150%);
            width: min(420px, calc(100vw - 16px));
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 12px;
            border-radius: 12px;
            background: #202c33;
            color: #e9edef;
            box-shadow: 0 6px 20px rgba(0, 0, 0, 0.35);
            font: 14px/1.3 "Segoe UI", sans-serif;
            cursor: pointer;
            z-index: 10001;
            transition: transform 0.2s ease-out;
        }
        #${POPUP_ID}.wl-show { transform: translate(-50%, 0); }
        #${POPUP_ID} img { width: 40px; height: 40px; border-radius: 50%; object-fit: cover; flex: none; }
        #${POPUP_ID} .wl-text { min-width: 0; flex: 1; }
        #${POPUP_ID} .wl-name { font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        #${POPUP_ID} .wl-preview { color: #aebac1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        #${POPUP_ID} .wl-close {
            flex: none; border: none; background: none; color: #aebac1;
            font-size: 18px; cursor: pointer; padding: 0 4px;
        }
    `;

    let counts = new Map();
    let observer = null;
    let scanTimer = null;
    let hideTimer = null;
    let started = false;

    function readRows() {
        const rows = new Map();
        const pane = document.getElementById('pane-side');
        if (!pane) return rows;
        for (const row of pane.querySelectorAll('[role="row"]')) {
            const name = row.querySelector('[data-testid="cell-frame-title"] span[title]');
            if (!name) continue;
            const badge = row.querySelector('[data-testid="icon-unread-count"]');
            rows.set(name.getAttribute('title'), {
                row,
                count: badge ? parseInt(badge.textContent, 10) || 1 : 0,
                open: row.matches('[aria-selected="true"]') || !!row.querySelector('[aria-selected="true"]'),
            });
        }
        return rows;
    }

    function inConversation() {
        return document.documentElement.classList.contains('wl-chat') && document.hasFocus();
    }

    function openChatName() {
        const title = document.querySelector('#main [data-testid="conversation-info-header-chat-title"]');
        return title ? title.textContent.trim() : null;
    }

    function scan() {
        scanTimer = null;
        const rows = readRows();
        if (started && inConversation()) {
            // a conversa aberta chega a ganhar o contador antes de ser marcada como lida
            const openName = openChatName();
            for (const [name, info] of rows) {
                if (info.open || name === openName) continue;
                if (info.count > (counts.get(name) || 0)) {
                    show(name, info);
                    break;
                }
            }
        }
        started = true;
        counts = new Map([...rows].map(([name, info]) => [name, info.count]));
    }

    function show(name, info) {
        const popup = document.getElementById(POPUP_ID);
        if (!popup) return;
        const status = info.row.querySelector('[data-testid="last-msg-status"]');
        const preview = status && (status.getAttribute('title')
            || (status.querySelector('span[dir]') || {}).textContent);
        const avatar = info.row.querySelector('img');

        popup.replaceChildren();
        if (avatar && avatar.src) {
            const img = document.createElement('img');
            img.src = avatar.src;
            popup.appendChild(img);
        }
        const text = document.createElement('div');
        text.className = 'wl-text';
        const title = document.createElement('div');
        title.className = 'wl-name';
        title.textContent = name;
        const body = document.createElement('div');
        body.className = 'wl-preview';
        body.textContent = preview || 'Nova mensagem';
        text.append(title, body);
        const close = document.createElement('button');
        close.className = 'wl-close';
        close.textContent = '×';
        close.title = 'Fechar';
        popup.append(text, close);

        popup.onclick = (e) => {
            hide();
            if (e.target === close) return;
            // o row muda de elemento quando a lista re-renderiza
            const current = readRows().get(name);
            const cell = current && current.row.querySelector('[role="gridcell"][tabindex]');
            if (cell) cell.click();
        };

        popup.classList.add('wl-show');
        clearTimeout(hideTimer);
        hideTimer = setTimeout(hide, SHOW_MS);
    }

    function hide() {
        const popup = document.getElementById(POPUP_ID);
        if (popup) popup.classList.remove('wl-show');
    }

    function scheduleScan() {
        if (!scanTimer) scanTimer = setTimeout(scan, 300);
    }

    function init() {
        const style = document.createElement('style');
        style.id = POPUP_ID + '-style';
        style.textContent = CSS;
        document.head.appendChild(style);
        const popup = document.createElement('div');
        popup.id = POPUP_ID;
        document.body.appendChild(popup);

        observer = new MutationObserver(scheduleScan);
        observer.observe(document.body, { childList: true, subtree: true, characterData: true });
        scheduleScan();
    }

    window.__wlPopup = {
        destroy() {
            if (observer) observer.disconnect();
            clearTimeout(scanTimer);
            clearTimeout(hideTimer);
            [POPUP_ID, POPUP_ID + '-style'].forEach(id => {
                const el = document.getElementById(id);
                if (el) el.remove();
            });
        }
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
