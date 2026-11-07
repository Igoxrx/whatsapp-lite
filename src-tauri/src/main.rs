#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::{AppHandle, Manager, Url, Webview, WebviewWindowBuilder, WindowEvent};

struct Settings {
    run_in_background: AtomicBool,
    ask_download_path: AtomicBool,
}

const DEFAULT_PROXY: Option<&str> = None;

const AD_REMOVAL_SCRIPT: &str = include_str!("../remove_ad.js");
const MOBILE_LAYOUT_SCRIPT: &str = include_str!("../mobile_layout.js");
const COMPACT_CHAT_SCRIPT: &str = include_str!("../compact_chat.js");

const WHATSAPP_ORIGIN: &str = "https://web.whatsapp.com/";

fn configure_webview_args() {
    let mut args = Vec::new();

    if let Some(proxy) = env::var("WHATSAPP_PROXY")
        .ok()
        .or_else(|| DEFAULT_PROXY.map(String::from))
    {
        args.push(format!("--proxy-server={}", proxy));
    }

    args.extend([
        "--disable-gpu-compositing".to_string(),
        "--disable-smooth-scrolling".to_string(),
        "--disable-features=msWebOOUI".to_string(),
    ]);

    if !args.is_empty() {
        env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", args.join(" "));
    }
}

fn open_in_browser(url: &Url) {
    // links de mensagens: nada de file://, ms-settings: etc.
    if !matches!(url.scheme(), "http" | "https") {
        return;
    }

    if let Err(e) = open::that_detached(url.as_str()) {
        eprintln!("Erro ao abrir link no navegador: {}", e);
    }
}

// O Tauri só trata a permissão de clipboard, então o WebView2 negava as
// notificações (e salvava a recusa no perfil).
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

fn settings_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join("settings.json"))
}

fn load_settings(app: &AppHandle) -> Settings {
    let json = settings_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .unwrap_or_default();
    let flag = |key: &str| AtomicBool::new(json[key].as_bool().unwrap_or(true));

    Settings {
        run_in_background: flag("run_in_background"),
        ask_download_path: flag("ask_download_path"),
    }
}

fn save_settings(app: &AppHandle) {
    let Some(path) = settings_path(app) else { return };
    let settings = app.state::<Settings>();
    let json = serde_json::json!({
        "run_in_background": settings.run_in_background.load(Ordering::Relaxed),
        "ask_download_path": settings.ask_download_path.load(Ordering::Relaxed),
    });
    let result = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|_| fs::write(&path, json.to_string()));
    if let Err(e) = result {
        eprintln!("Erro ao salvar preferências: {}", e);
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        set_in_tray(&window, false);
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

// Na bandeja o WebView2 para de desenhar e libera memória; o JavaScript
// continua rodando, então mensagens e notificações seguem chegando.
fn set_in_tray(window: &tauri::WebviewWindow, in_tray: bool) {
    #[cfg(windows)]
    let _ = window.with_webview(move |webview| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::*;
        use windows::core::Interface;

        let controller = webview.controller();
        let _ = controller.SetIsVisible(!in_tray);
        let level = if in_tray {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
        } else {
            COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
        };
        if let Ok(core) = controller
            .CoreWebView2()
            .and_then(|core| core.cast::<ICoreWebView2_19>())
        {
            let _ = core.SetMemoryUsageTargetLevel(level);
        }
    });
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let settings = app.state::<Settings>();

    let open = MenuItem::with_id(app, "open", "Abrir", true, None::<&str>)?;
    let background = CheckMenuItem::with_id(
        app,
        "background",
        "Rodar em segundo plano",
        true,
        settings.run_in_background.load(Ordering::Relaxed),
        None::<&str>,
    )?;
    let ask_download = CheckMenuItem::with_id(
        app,
        "ask_download",
        "Perguntar onde salvar downloads",
        true,
        settings.ask_download_path.load(Ordering::Relaxed),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &background, &ask_download, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("WhatsApp Lite")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            // o clique já alternou o check
            let toggled = match event.id().as_ref() {
                "open" => return show_main_window(app),
                "quit" => return app.exit(0),
                "background" => (&background, &app.state::<Settings>().run_in_background),
                "ask_download" => (&ask_download, &app.state::<Settings>().ask_download_path),
                _ => return,
            };
            let (item, flag) = toggled;
            flag.store(item.is_checked().unwrap_or(true), Ordering::Relaxed);
            save_settings(app);
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

/// Janela "Salvar como" partindo do nome e da pasta que o WebView2 sugeriu.
/// `None` quando o usuário cancela.
fn ask_save_path(webview: &Webview, suggested: &Path) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_parent(&webview.window());
    if let Some(dir) = suggested.parent() {
        dialog = dialog.set_directory(dir);
    }
    if let Some(name) = suggested.file_name() {
        dialog = dialog.set_file_name(name.to_string_lossy());
    }
    if let Some(ext) = suggested.extension().and_then(|ext| ext.to_str()) {
        dialog = dialog.add_filter(ext.to_uppercase(), &[ext]);
    }
    dialog.save_file()
}

fn on_download(webview: Webview, event: DownloadEvent<'_>) -> bool {
    if let DownloadEvent::Requested { destination, .. } = event {
        let settings = webview.app_handle().state::<Settings>();
        if settings.ask_download_path.load(Ordering::Relaxed) {
            match ask_save_path(&webview, destination) {
                Some(path) => *destination = path,
                None => return false,
            }
        }
    }
    true
}

fn setup_main_window(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config = app
        .config()
        .app
        .windows
        .first()
        .ok_or("Falha ao obter configuração da janela principal")?;

    let window = WebviewWindowBuilder::from_config(app, config)?
        .initialization_script(AD_REMOVAL_SCRIPT)
        .initialization_script(MOBILE_LAYOUT_SCRIPT)
        .initialization_script(COMPACT_CHAT_SCRIPT)
        .on_new_window(|url, _features| {
            open_in_browser(&url);
            NewWindowResponse::Deny
        })
        .on_download(on_download)
        // deixa o WhatsApp receber os arquivos arrastados para a janela
        .disable_drag_drop_handler()
        .build()?;

    #[cfg(windows)]
    allow_notifications(&window)?;

    Ok(())
}

fn main() {
    configure_webview_args();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .setup(|app| {
            app.manage(load_settings(app.handle()));

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
                    if let Some(webview) = window.app_handle().get_webview_window(window.label()) {
                        set_in_tray(&webview, true);
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar WhatsApp Lite Desktop");
}
