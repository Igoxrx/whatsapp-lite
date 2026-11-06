/**
 * WhatsApp Lite Desktop - Layout estilo celular
 *
 * Sem conversa aberta, a lista (com a barra de conversas, status, canais e
 * comunidades) ocupa a janela toda. Ao abrir uma conversa, só o chat aparece,
 * com um botão de voltar no canto superior esquerdo.
 */

(function() {
    'use strict';

    const STYLE_ID = 'wl-mobile-layout';
    const BACK_ID = 'wl-back-button';

    // Permite reinjetar o script sem duplicar o botão
    if (window.__wlMobile) window.__wlMobile.destroy();

    // Abaixo desta largura, a barra lateral (conversas, status, canais...) vai para baixo
    const NARROW_WIDTH = 640;
    const BOTTOM_NAV_HEIGHT = 56;

    let timer = null;

    const CSS = `
        @media (max-width: ${NARROW_WIDTH}px) {
            html.wl-list [data-wl="nav"] {
                position: absolute !important;
                top: auto !important;
                bottom: 0 !important;
                left: 0 !important;
                right: 0 !important;
                width: auto !important;
                height: ${BOTTOM_NAV_HEIGHT}px !important;
                padding: 8px 12px !important;
                z-index: 5;
            }
            html.wl-list [data-wl="nav"] > div {
                flex-direction: row !important;
                justify-content: space-between !important;
                align-items: center !important;
                width: 100% !important;
                height: 100% !important;
            }
            html.wl-list [data-wl="nav"] > div > div {
                display: flex !important;
                flex: 0 0 auto !important;
                width: auto !important;
                height: auto !important;
            }
            html.wl-list [data-wl="nav"] > div > div > div {
                flex-direction: row !important;
                align-items: center !important;
                gap: 4px;
                height: auto !important;
            }
            html.wl-list [data-wl="nav"] hr { display: none !important; }
            html.wl-list [data-wl="col"] {
                margin-left: 0 !important;
                height: calc(100% - ${BOTTOM_NAV_HEIGHT}px) !important;
            }
            html.wl-list [data-wl="layer"] {
                left: 0 !important;
                right: 0 !important;
                width: auto !important;
                bottom: ${BOTTOM_NAV_HEIGHT}px !important;
                height: auto !important;
            }
        }

        html.wl-list [data-wl="col"] {
            flex: 1 1 auto !important;
            width: auto !important;
            max-width: none !important;
        }
        html.wl-list [data-wl="main"] { display: none !important; }

        html.wl-chat [data-wl="nav"],
        html.wl-chat [data-wl="col"] { display: none !important; }
        html.wl-chat [data-wl="main"] {
            flex: 1 1 auto !important;
            width: auto !important;
            max-width: none !important;
        }
        html.wl-chat [data-testid="conversation-header"] { padding-left: 52px !important; }

        #${BACK_ID} {
            position: fixed;
            top: 12px;
            left: 8px;
            width: 40px;
            height: 40px;
            border: none;
            border-radius: 50%;
            background: transparent;
            color: inherit;
            cursor: pointer;
            z-index: 10000;
            display: none;
            align-items: center;
            justify-content: center;
        }
        #${BACK_ID}:hover { background: rgba(127, 127, 127, 0.2); }
        html.wl-chat #${BACK_ID} { display: flex; }
    `;

    /**
     * Encontra a coluna que contém a lista de conversas: o filho do layout
     * principal (.two) que contém #side.
     */
    function findColumn() {
        let el = document.getElementById('side');
        if (!el) return null;
        while (el.parentElement && !el.parentElement.classList.contains('two')) {
            el = el.parentElement;
            if (el === document.body) return null;
        }
        return el.parentElement ? el : null;
    }

    /**
     * Pares [lista, painel] que seguem o mesmo layout: a coluna principal e
     * as camadas sobrepostas do WhatsApp (painéis, transições).
     */
    function findPairs(column) {
        const pairs = [[column, column.nextElementSibling]];
        const left = column.offsetLeft;
        for (const layer of column.parentElement.children) {
            if (layer === column || getComputedStyle(layer).position !== 'absolute') continue;
            for (const child of layer.children) {
                const next = child.nextElementSibling;
                if (next && Math.abs(child.offsetLeft + layer.offsetLeft - left) <= 1 &&
                    child.offsetWidth < layer.offsetWidth) {
                    pairs.push([child, next]);
                    break;
                }
            }
        }
        return pairs;
    }

    // Só escreve quando muda: cada escrita faz o navegador recalcular a tela
    function tag(el, value) {
        if (el && el.getAttribute('data-wl') !== value) el.setAttribute('data-wl', value);
    }

    function setMode(mode) {
        const root = document.documentElement;
        for (const m of ['wl-list', 'wl-chat']) {
            const on = m === mode;
            if (root.classList.contains(m) !== on) root.classList.toggle(m, on);
        }
    }

    /**
     * Há algo aberto no painel da direita (conversa, canal etc.) quando ele
     * tem conteúdo e não é a tela inicial vazia do WhatsApp.
     */
    function paneHasContent(pane) {
        if (!pane || !pane.firstElementChild) return false;
        return !pane.querySelector('[data-testid="intro-panel"]');
    }

    function refresh() {
        const column = findColumn();
        if (!column) {
            setMode(null);
            return;
        }
        const nav = column.parentElement.querySelector(':scope > header');
        tag(nav, 'nav');
        // Camadas recuadas para dar lugar à barra lateral (painéis que abrem
        // por cima da lista); com a barra embaixo, o recuo é removido
        for (const layer of column.parentElement.children) {
            const style = getComputedStyle(layer);
            if (style.position === 'absolute' && parseFloat(style.left) > 0) tag(layer, 'layer');
        }
        tag(column, 'col');
        tag(column.nextElementSibling, 'main');
        // As camadas só podem ser medidas com a lista visível
        if (column.offsetWidth > 0) {
            for (const [list, pane] of findPairs(column)) {
                tag(list, 'col');
                tag(pane, 'main');
            }
        }
        setMode(paneHasContent(column.nextElementSibling) ? 'wl-chat' : 'wl-list');
    }

    /**
     * Volta para a lista: o WhatsApp fecha a conversa aberta com Esc.
     */
    function goBack() {
        const init = { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true, cancelable: true };
        (document.activeElement || document.body).dispatchEvent(new KeyboardEvent('keydown', init));
        setTimeout(refresh, 100);
    }

    function createBackButton() {
        const button = document.createElement('button');
        button.id = BACK_ID;
        button.title = 'Voltar';
        button.innerHTML = '<svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">' +
            '<path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/></svg>';
        button.addEventListener('click', goBack);
        document.body.appendChild(button);
    }

    function init() {
        const style = document.createElement('style');
        style.id = STYLE_ID;
        style.textContent = CSS;
        document.head.appendChild(style);
        createBackButton();
        refresh();
        timer = setInterval(refresh, 300);
    }

    window.__wlMobile = {
        destroy() {
            clearInterval(timer);
            setMode(null);
            document.querySelectorAll('[data-wl]').forEach(el => el.removeAttribute('data-wl'));
            [STYLE_ID, BACK_ID].forEach(id => {
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
