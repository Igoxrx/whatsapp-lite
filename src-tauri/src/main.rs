//! # WhatsApp Lite Desktop
//!
//! Uma versão leve do WhatsApp Desktop feita em Rust usando Tauri.
//!
//! ## Motivação
//! O WhatsApp Desktop oficial baseado em Electron consome recursos excessivos,
//! chegando a usar 60% da GPU em algumas máquinas. Esta versão alternativa
//! usa apenas ~1MB de RAM e praticamente zero GPU.
//!
//! ## Arquitetura
//! - **Backend**: Rust + Tauri 2.0
//! - **Frontend**: WebView nativo (WebView2 no Windows)
//! - **Injeção de scripts**: Remove anúncios e elementos desnecessários

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Url, WebviewWindowBuilder, WindowEvent};

/// Preferências do usuário que valem entre execuções
struct Settings {
    /// Fechar a janela só a esconde; o app continua na bandeja do sistema
    run_in_background: AtomicBool,
}

/// Configuração padrão do proxy (pode ser sobrescrita via variável de ambiente)
const DEFAULT_PROXY: Option<&str> = None;

/// Script JavaScript para remover anúncios e otimizar a interface
const AD_REMOVAL_SCRIPT: &str = include_str!("../remove_ad.js");

/// Script JavaScript do layout estilo celular (lista ou conversa, com botão de voltar)
const MOBILE_LAYOUT_SCRIPT: &str = include_str!("../mobile_layout.js");

/// Única origem que pode mostrar notificações
const WHATSAPP_ORIGIN: &str = "https://web.whatsapp.com/";

/// Script JavaScript que reduz as margens das mensagens
const COMPACT_CHAT_SCRIPT: &str = include_str!("../compact_chat.js");

/// Configura argumentos adicionais do WebView2 para otimização
fn configure_webview_args() {
    let mut args = Vec::new();

    // Configuração de proxy (opcional)
    if let Some(proxy) = env::var("WHATSAPP_PROXY")
        .ok()
        .or_else(|| DEFAULT_PROXY.map(String::from))
    {
        args.push(format!("--proxy-server={}", proxy));
    }

    // Otimizações de performance
    args.extend([
        "--disable-gpu-compositing".to_string(),      // Reduz uso de GPU
        "--disable-smooth-scrolling".to_string(),     // Menos overhead de animação
        "--disable-features=msWebOOUI".to_string(),   // Remove UI desnecessária
    ]);

    if !args.is_empty() {
        env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args.join(" "));
    }
}

/// Abre um link no navegador padrão do sistema
fn open_in_browser(url: &Url) {
    // Só http/https: uma mensagem não pode disparar outros esquemas no sistema
    if !matches!(url.scheme(), "http" | "https") {
        return;
    }

    if let Err(e) = open::that_detached(url.as_str()) {
        eprintln!("Erro ao abrir link no navegador: {}", e);
    }
}

/// Permite que o WhatsApp Web mostre notificações de mensagens.
///
/// O Tauri só trata a permissão de área de transferência; sem isso o WebView2
/// recusa o pedido de notificação na hora e guarda a recusa no perfil.
#[cfg(windows)]
fn allow_notifications(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use webview2_com::{
        take_pwstr, PermissionRequestedEventHandler, SetPermissionStateCompletedHandler,
    };
    use windows::core::{w, Interface};

    window.with_webview(|webview| unsafe {
        let Ok(core) = webview.controller().CoreWebView2() else {
            return;
        };

        // Substitui uma recusa já salva no perfil (de versões anteriores do app)
        let profile = core
            .cast::<ICoreWebView2_13>()
            .and_then(|core| core.Profile())
            .and_then(|profile| profile.cast::<ICoreWebView2Profile4>());
        if let Ok(profile) = profile {
            let done = SetPermissionStateCompletedHandler::create(Box::new(|result| {
                if let Err(e) = result {
                    eprintln!("Erro ao liberar notificações: {}", e);
                }
                Ok(())
            }));
            let _ = profile.SetPermissionState(
                COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS,
                w!("https://web.whatsapp.com"),
                COREWEBVIEW2_PERMISSION_STATE_ALLOW,
                &done,
            );
        }

        let handler = PermissionRequestedEventHandler::create(Box::new(|_, args| {
            let Some(args) = args else { return Ok(()) };

            let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
            args.PermissionKind(&mut kind)?;
            if kind != COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS {
                return Ok(());
            }

            let mut uri = Default::default();
            args.Uri(&mut uri)?;
            if take_pwstr(uri).starts_with(WHATSAPP_ORIGIN) {
                args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
            }

            Ok(())
        }));

        let mut token = 0;
        if let Err(e) = core.add_PermissionRequested(&handler, &mut token) {
            eprintln!("Erro ao configurar notificações: {}", e);
        }
    })
}

/// Arquivo onde as preferências ficam salvas
fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

/// Lê se o app deve rodar em segundo plano (padrão: sim)
fn load_run_in_background(app: &AppHandle) -> bool {
    settings_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|json| json["run_in_background"].as_bool())
        .unwrap_or(true)
}

fn save_run_in_background(app: &AppHandle, value: bool) {
    let Some(path) = settings_path(app) else { return };
    let json = serde_json::json!({ "run_in_background": value });
    let result = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|_| fs::write(&path, json.to_string()));
    if let Err(e) = result {
        eprintln!("Erro ao salvar preferências: {}", e);
    }
}

/// Mostra e foca a janela principal (ex.: depois de escondida na bandeja)
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Ícone na bandeja do sistema com as opções Abrir, Rodar em segundo plano e Sair
fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let run_in_background = app.state::<Settings>().run_in_background.load(Ordering::Relaxed);

    let open = MenuItem::with_id(app, "open", "Abrir", true, None::<&str>)?;
    let background = CheckMenuItem::with_id(
        app,
        "background",
        "Rodar em segundo plano",
        true,
        run_in_background,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &background, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("WhatsApp Lite")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "background" => {
                // O item já foi marcado/desmarcado pelo clique
                let enabled = background.is_checked().unwrap_or(true);
                app.state::<Settings>()
                    .run_in_background
                    .store(enabled, Ordering::Relaxed);
                save_run_in_background(app, enabled);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;

    Ok(())
}

/// Cria a janela principal com scripts de otimização
fn setup_main_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config = app
        .config()
        .app
        .windows
        .first()
        .ok_or("Falha ao obter configuração da janela principal")?;

    // Scripts de inicialização rodam a cada carregamento da página
    let window = WebviewWindowBuilder::from_config(app, config)?
        .initialization_script(AD_REMOVAL_SCRIPT)
        .initialization_script(MOBILE_LAYOUT_SCRIPT)
        .initialization_script(COMPACT_CHAT_SCRIPT)
        .on_new_window(|url, _features| {
            open_in_browser(&url);
            NewWindowResponse::Deny
        })
        .build()?;

    #[cfg(windows)]
    allow_notifications(&window)?;

    Ok(())
}

fn main() {
    // Configura otimizações do WebView antes de iniciar
    configure_webview_args();

    tauri::Builder::default()
        // Abrir o app de novo só mostra a janela que já está rodando
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .setup(|app| {
            let run_in_background = load_run_in_background(app.handle());
            app.manage(Settings {
                run_in_background: AtomicBool::new(run_in_background),
            });

            setup_main_window(app).map_err(|e| {
                eprintln!("Erro na inicialização: {}", e);
                e
            })?;
            setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let settings = window.app_handle().state::<Settings>();
                if settings.run_in_background.load(Ordering::Relaxed) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar WhatsApp Lite Desktop");
}
