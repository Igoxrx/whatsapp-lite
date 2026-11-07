(function() {
    'use strict';

    const AD_SELECTORS = [
        '._al_d',
        '[data-testid="banner-upgrade"]',
        '[data-testid="download-app-banner"]',
        '.x1c4vz4f.xs83m0k.xdl72j9.x1g77sc7.xozqiw3.x1oa3qoh.x12fk4p8.xeuugli.x2lwn1j.xaw8158.x1q0g3np.x6s0dn4.x182zwpg.x3nfvp2'
    ];

    const PERFORMANCE_TWEAKS = {
        // ligado, a regra * deixa a thread principal 100% em recalc de estilo
        // e as mensagens chegam com atraso
        disableAnimations: false,
        reduceBgProcessing: true
    };

    function removeAds() {
        AD_SELECTORS.forEach(selector => {
            try {
                const elements = document.querySelectorAll(selector);
                elements.forEach(el => {
                    if (el && el.parentNode) {
                        el.remove();
                        console.debug('[WhatsApp Lite] Removed:', selector);
                    }
                });
            } catch (e) {}
        });
        hideDownloadBanner();
    }

    // as classes desse aviso mudam toda hora, então vai pelo texto
    function hideDownloadBanner() {
        const side = document.getElementById('side');
        if (!side) return;
        for (const child of side.children) {
            if (child.style.display === 'none') continue;
            if (child.querySelector('[role="grid"], [role="listbox"]')) continue;
            if (/WhatsApp (para|for) Windows/i.test(child.textContent)) {
                child.style.display = 'none';
            }
        }
    }

    function applyPerformanceTweaks() {
        if (PERFORMANCE_TWEAKS.disableAnimations) {
            const style = document.createElement('style');
            style.textContent = `
                *, *::before, *::after {
                    animation-duration: 0.01ms !important;
                    animation-iteration-count: 1 !important;
                    transition-duration: 0.01ms !important;
                }
            `;
            document.head.appendChild(style);
        }
    }

    function initObserver() {
        const observer = new MutationObserver((mutations) => {
            if (!initObserver.timeout) {
                initObserver.timeout = setTimeout(() => {
                    removeAds();
                    initObserver.timeout = null;
                }, 100);
            }
        });

        observer.observe(document.body, {
            childList: true,
            subtree: true
        });

        return observer;
    }

    function init() {
        removeAds();
        applyPerformanceTweaks();
        initObserver();
        console.info('[WhatsApp Lite] Otimizações aplicadas com sucesso!');
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
