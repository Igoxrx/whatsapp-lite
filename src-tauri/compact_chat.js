(function() {
    'use strict';

    const STYLE_ID = 'wl-compact-chat';

    const ROW_PADDING = '8px'; // original: 63px
    const BUBBLE_MAX_WIDTH = '95%'; // original: 65% a 95%

    // .two vem com min-width: 748px
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
