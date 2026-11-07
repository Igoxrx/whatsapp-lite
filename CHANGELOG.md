# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [1.2.0] - 2026-11-07

### Adicionado
- Downloads abrem a janela "Salvar como", com opção no menu da bandeja para desligar
- Arquivos arrastados para a janela vão para a conversa
- Na bandeja, o app usa bem menos memória (cerca de 1,1 GB caem para 360 MB)

### Corrigido
- Conversa abria no topo em vez de na mensagem mais recente
- Era preciso clicar na caixa de mensagem para começar a digitar
- Na aba Status, não dava para voltar para Conversas

## [1.1.0] - 2026-11-07

### Adicionado
- Layout estilo celular: lista em tela cheia e conversa com botão de voltar
- Barra de navegação embaixo em janelas com menos de 640 px
- Chat compacto, com margens menores e balões mais largos
- Segundo plano na bandeja do sistema, com opção para ligar ou desligar
- Uma instância só: abrir de novo mostra a janela existente
- Notificações de mensagens no Windows
- Links abrem no navegador padrão (só http/https)
- Botão voltar do mouse fecha a conversa
- Esconde o aviso "Baixar o WhatsApp para Windows"

### Corrigido
- Scripts eram injetados antes de a página carregar e não rodavam
- Regra de animações deixava a página 100% ocupada e atrasava as mensagens
- Largura mínima de 748 px impedia o chat de encolher com a janela

### Removido
- Tamanho mínimo da janela (800×500)
- Alça para redimensionar a lista de conversas, substituída pelo layout estilo celular

## [1.0.0] - 2026-01-17

### Adicionado
- 🎉 Lançamento inicial do WhatsApp Lite Desktop
- 🦀 Backend em Rust com Tauri 2.0
- 🪶 Uso de WebView nativo (WebView2 no Windows)
- 🚫 Script automático para remoção de anúncios
- ⚡ Otimizações de GPU desabilitando composição desnecessária
- 🔧 Suporte a proxy via variável de ambiente `WHATSAPP_PROXY`
- 📱 Ícone na bandeja do sistema
- 🖥️ Suporte a Windows, macOS e Linux
- 📄 Documentação completa
- 🌐 Landing page explicativa

### Otimizações
- Uso de ~1MB de RAM (vs 500MB+ do oficial)
- GPU praticamente zerada (vs 60%+ do oficial)
- Instalador de ~3MB (vs ~150MB do oficial)
- Startup < 1 segundo (vs 5-10s do oficial)

### Técnico
- Configuração de CSP para segurança
- Profile de release otimizado (LTO, strip, opt-level z)
- Debounce no observer de DOM para performance
- Desabilitação de animações CSS para reduzir overhead
