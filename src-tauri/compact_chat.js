/**
 * WhatsApp Lite Desktop - Chat compacto
 *
 * Reduz as margens laterais das mensagens e deixa os balões ocuparem
 * mais da largura do chat, aproveitando melhor o espaço da janela.
 */

(function() {
    'use strict';

    const STYLE_ID = 'wl-compact-chat';

    // Margem lateral das linhas de mensagem (o WhatsApp usa 63px)
    const ROW_PADDING = '8px';

    // Largura máxima dos balões (o WhatsApp usa 65% a 95% conforme a janela)
    const BUBBLE_MAX_WIDTH = '95%';

    // .two é o layout principal, com largura mínima de 748px que impede o
    // chat de encolher em janelas estreitas.
    // Classes atômicas do WhatsApp Web: cada uma corresponde a uma única
    // declaração CSS, então só mudam se o WhatsApp mudar esses valores
    const CSS = `
        .two { min-width: 0 !important; }
        #main .x1wxbj4f { padding-inline-start: ${ROW_PADDING} !important; }
        #main .xh1vicy { padding-inline-end: ${ROW_PADDING} !important; }
        #main .xb3dwbw, #main .x2nxwck, #main .x14fraq8, #main .x1gx27l9 {
            max-width: ${BUBBLE_MAX_WIDTH} !important;
        }
    `;

    function init() {
        let style = document.getElementById(STYLE_ID);
        if (!style) {
            style = document.createElement('style');
            style.id = STYLE_ID;
            document.head.appendChild(style);
        }
        style.textContent = CSS;
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
