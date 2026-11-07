<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="WhatsApp Lite" width="96" height="96">
</p>

<h1 align="center">WhatsApp Lite</h1>

<p align="center">
  WhatsApp Web num app leve para Windows, feito com Tauri e Rust.
</p>

Fork do [WhatsApp Lite Desktop](https://github.com/leonardo-matheus/whatsapp-lite), de Leonardo Matheus, com layout estilo celular, bandeja do sistema, notificações e outras melhorias.

O app abre o WhatsApp Web oficial no WebView2 do Windows, em vez de embutir um navegador inteiro como o WhatsApp Desktop baseado em Electron. Por isso o instalador tem menos de 1 MB.

## O que tem

- **Layout estilo celular:** a lista de conversas ocupa a janela. Ao abrir uma conversa, só o chat aparece, com um botão de voltar.
- **Barra embaixo em janelas estreitas:** abaixo de 640 px, os botões de Conversas, Status, Canais e Comunidades vão para o rodapé.
- **Chat compacto:** margens menores e balões mais largos, e a janela pode ter qualquer tamanho.
- **Segundo plano:** fechar a janela deixa o app no ícone perto do relógio, onde dá para ligar ou desligar essa opção. Na bandeja, o app usa bem menos memória (cerca de 360 MB, contra mais de 1 GB com a janela aberta).
- **Notificações** de mensagens no Windows.
- **Downloads com "Salvar como":** você escolhe a pasta e o nome do arquivo.
- **Arrastar arquivos** para a conversa.
- **Links** abrem no seu navegador padrão.
- **Botão voltar do mouse** e a tecla Esc fecham a conversa.
- **Aviso "Baixar o WhatsApp para Windows"** escondido.
- **Uma instância só:** abrir o app de novo mostra a janela que já está rodando.

## Instalação

1. Baixe o `WhatsApp.Lite_1.2.0_x64-setup.exe` em [Releases](https://github.com/Igoxrx/whatsapp-lite/releases/latest).
2. Abra o instalador. Ele não tem assinatura digital, então o Windows pode mostrar "O Windows protegeu o computador". Clique em **Mais informações** e depois em **Executar assim mesmo**.
3. Abra o WhatsApp Lite e leia o QR code com o celular, como no WhatsApp Web.

O app precisa do WebView2, que já vem no Windows 11. Se faltar, o instalador baixa.

## Bandeja do sistema

Clique com o botão direito no ícone perto do relógio:

- **Abrir:** mostra a janela.
- **Rodar em segundo plano:** marcado, o X só esconde a janela. Desmarcado, o X fecha o app.
- **Perguntar onde salvar downloads:** marcado, abre a janela "Salvar como". Desmarcado, os arquivos vão direto para a pasta Downloads.
- **Sair:** fecha o app.

As escolhas ficam salvas em `%APPDATA%\com.whatsapp-lite.desktop\settings.json`.

## Compilar

Requisitos: [Rust](https://rustup.rs/) e [Node.js](https://nodejs.org/) (só para a CLI do Tauri).

```bash
git clone https://github.com/Igoxrx/whatsapp-lite.git
cd whatsapp-lite/src-tauri
npx @tauri-apps/cli@2 build --bundles nsis
```

O instalador fica em `src-tauri/target/release/bundle/nsis/`, e o executável em `src-tauri/target/release/whatsapp-lite.exe`.

Os ajustes de layout ficam em scripts injetados na página:

| Arquivo | O que faz |
|---|---|
| `src-tauri/mobile_layout.js` | Layout estilo celular, barra embaixo e botão de voltar |
| `src-tauri/compact_chat.js` | Margens e largura das mensagens |
| `src-tauri/remove_ad.js` | Esconde anúncios e o aviso de download |

O WhatsApp Web muda a página com frequência. Se algum ajuste parar de funcionar depois de uma atualização dele, é nesses arquivos que se corrige.

## Proxy

Para usar um proxy, defina a variável de ambiente antes de abrir o app:

```powershell
$env:WHATSAPP_PROXY = "http://seu-proxy:porta"
```

## Aviso

Este é um cliente não oficial. Ele não tem ligação com o WhatsApp ou com a Meta e só carrega o WhatsApp Web oficial.

## Licença

MIT, como o projeto original. Veja [LICENSE](LICENSE).
