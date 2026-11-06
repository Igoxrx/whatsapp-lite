/**
 * WhatsApp Lite Desktop - Script de Otimização
 * 
 * Remove elementos desnecessários e anúncios da interface do WhatsApp Web
 * para melhorar performance e reduzir consumo de recursos.
 */

(function() {
    'use strict';

    // Seletores de elementos a serem removidos (anúncios, banners, etc.)
    const AD_SELECTORS = [
        '._al_d',
        '[data-testid="banner-upgrade"]',
        '[data-testid="download-app-banner"]',
        '.x1c4vz4f.xs83m0k.xdl72j9.x1g77sc7.xozqiw3.x1oa3qoh.x12fk4p8.xeuugli.x2lwn1j.xaw8158.x1q0g3np.x6s0dn4.x182zwpg.x3nfvp2'
    ];

    // Seletores para otimização de performance
    const PERFORMANCE_TWEAKS = {
        // Desligado: a regra universal deixava a thread principal 100% ocupada
        // recalculando estilos e atrasava o envio e o recebimento de mensagens
        disableAnimations: false,
        reduceBgProcessing: true
    };

    /**
     * Remove elementos de anúncio do DOM
     */
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
            } catch (e) {
                // Seletor inválido, ignorar silenciosamente
            }
        });
        hideDownloadBanner();
    }

    /**
     * Esconde o aviso "Baixar o WhatsApp para Windows" no fim da lista de
     * conversas. As classes dele mudam com frequência, então ele é achado
     * pelo texto, só entre os blocos pequenos de #side (nunca a lista em si).
     */
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

    /**
     * Aplica otimizações de CSS para reduzir animações
     */
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

    /**
     * Inicializa o observer para monitorar mudanças no DOM
     */
    function initObserver() {
        const observer = new MutationObserver((mutations) => {
            // Debounce para evitar chamadas excessivas
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

    // Inicialização
    function init() {
        removeAds();
        applyPerformanceTweaks();
        initObserver();
        console.info('[WhatsApp Lite] Otimizações aplicadas com sucesso!');
    }

    // Aguarda o DOM estar pronto
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
