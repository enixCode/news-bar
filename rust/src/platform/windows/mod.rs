// Backend Windows : fenetre Win32 sans bordure, defilement des titres RSS, et
// reservation d'espace ecran (AppBar). Un thread worker recupere les flux et
// reconstruit le bandeau ; un clic gauche ouvre l'article sous le curseur.
//   - render : pre-rendu du bandeau + hit-boxes (titre -> url)
//   - appbar : reservation / liberation de l'espace ecran

mod appbar;
mod render;

use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::Config;
use crate::feeds::{fetch_all, Item};
use crate::translate::translate_titles;
use render::Strip;

use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, GetDC, GetDeviceCaps, InvalidateRect, ReleaseDC, LOGPIXELSY, PAINTSTRUCT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ};
use windows_sys::Win32::UI::HiDpi::{SetProcessDpiAwareness, PROCESS_SYSTEM_DPI_AWARE};
use windows_sys::Win32::UI::Shell::{ShellExecuteW, APPBARDATA};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetClientRect, GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowLongPtrW,
    KillTimer, LoadCursorW, PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow,
    SetTimer, SetWindowLongPtrW, ShowWindow, TrackPopupMenu, TranslateMessage, CREATESTRUCTW,
    GWLP_USERDATA, IDC_ARROW, MF_STRING, MSG, SM_CXSCREEN, SM_CYSCREEN, SW_SHOWNOACTIVATE,
    SW_SHOWNORMAL, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_DESTROY, WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP,
    WM_TIMER, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_VISIBLE,
};

const TIMER_ID: usize = 1;
const MENU_CLOSE_ID: usize = 1;
// Message custom : le worker previent l'UI qu'un nouveau lot de titres est pret.
const WM_APP_REFRESH: u32 = 0x8000 + 1;

// Etat partage : cree dans run(), pointe depuis la fenetre (GWLP_USERDATA),
// lu/ecrit par la WndProc. `pending` est rempli par le thread worker.
struct State {
    offset: f64,
    speed: f64,
    scale: f64,
    dpi: i32,
    screen_w: i32,
    screen_h: i32,
    is_bottom: bool,
    cfg: Config,
    strip: Strip,
    abd: APPBARDATA,
    abd_set: bool,
    pending: Arc<Mutex<Option<Vec<Item>>>>,
}

// Helpers partages avec les sous-modules (render, appbar).
pub(super) fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(super) fn rgb(c: [u8; 3]) -> COLORREF {
    (c[0] as u32) | ((c[1] as u32) << 8) | ((c[2] as u32) << 16)
}

// Inscrit news-bar au demarrage automatique (cle Run de l'utilisateur courant).
// Idempotent : reecrit le chemin de l'exe a chaque lancement.
unsafe fn register_autostart() {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => return,
    };
    let value = wide(&format!("\"{}\"", exe.to_string_lossy()));
    let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let name = wide("news-bar");
    let bytes = (value.len() * 2) as u32; // octets, terminateur nul inclus
    RegSetKeyValueW(
        HKEY_CURRENT_USER,
        subkey.as_ptr(),
        name.as_ptr(),
        REG_SZ,
        value.as_ptr() as *const c_void,
        bytes,
    );
}

// Thread de fond : fetch immediat puis toutes les `refresh_minutes`. A chaque
// lot non vide, depose le resultat et reveille l'UI. Reseau coupe -> on ne
// touche a rien (le dernier contenu reste affiche).
fn worker(hwnd_val: isize, cfg: Config, pending: Arc<Mutex<Option<Vec<Item>>>>) {
    // Cache titre original -> traduction, conserve entre les refresh : on ne
    // retraduit que les titres reellement nouveaux.
    let mut cache: HashMap<String, String> = HashMap::new();
    loop {
        let heads = fetch_all(&cfg.feeds, cfg.max_items_per_feed);
        if !heads.is_empty() {
            // Traduction optionnelle des titres absents du cache (1 appel API).
            if cfg.translate {
                if let Ok(key) = std::env::var("GROQ_API_KEY") {
                    let mut todo: Vec<String> = heads
                        .iter()
                        .map(|h| h.title.clone())
                        .filter(|t| !cache.contains_key(t))
                        .collect();
                    todo.sort();
                    todo.dedup();
                    if let Some(tr) = translate_titles(&todo, &key, &cfg.groq_model) {
                        for (o, f) in todo.into_iter().zip(tr.into_iter()) {
                            cache.insert(o, f);
                        }
                    }
                }
            }

            let items: Vec<Item> = heads
                .iter()
                .map(|h| {
                    let title = cache.get(&h.title).cloned().unwrap_or_else(|| h.title.clone());
                    Item {
                        text: cfg
                            .item_format
                            .replace("{source}", &h.source)
                            .replace("{title}", &title),
                        url: h.url.clone(),
                    }
                })
                .collect();

            *pending.lock().unwrap() = Some(items);
            unsafe { PostMessageW(hwnd_val as HWND, WM_APP_REFRESH, 0, 0) };
        }
        std::thread::sleep(Duration::from_secs(cfg.refresh_minutes.max(1) * 60));
    }
}

pub fn run(cfg: Config) {
    unsafe {
        SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);
        register_autostart();

        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = wide("NewsBarWindowClass");

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: ptr::null_mut(),
            hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
            hbrBackground: ptr::null_mut(), // pas d'effacement auto : on peint tout (anti-flicker)
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        // Echelle DPI : avec un process DPI-aware, LOGPIXELSY rend le vrai DPI.
        let screen_dc = GetDC(ptr::null_mut());
        let dpi = GetDeviceCaps(screen_dc, LOGPIXELSY as i32);
        ReleaseDC(ptr::null_mut(), screen_dc);
        let scale = dpi as f64 / 96.0;

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let is_bottom = cfg.position != "top";

        // Bandeau initial (placeholder) ; le worker le remplacera apres le 1er fetch.
        let strip = Strip::build(&cfg, dpi, scale, screen_w, &[]);
        let bar_height = strip.bar_height;

        let pending = Arc::new(Mutex::new(None));
        let state = Box::new(State {
            offset: 0.0,
            speed: cfg.speed,
            scale,
            dpi,
            screen_w,
            screen_h,
            is_bottom,
            cfg: cfg.clone(),
            strip,
            abd: std::mem::zeroed(),
            abd_set: false,
            pending: pending.clone(),
        });
        let state_ptr = Box::into_raw(state);

        // Position provisoire : recalee dans WM_CREATE apres reponse de l'AppBar.
        let init_y = if is_bottom { screen_h - bar_height } else { 0 };

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            wide("news-bar").as_ptr(),
            WS_POPUP | WS_VISIBLE,
            0,
            init_y,
            screen_w,
            bar_height,
            ptr::null_mut(),
            ptr::null_mut(),
            hinstance,
            state_ptr as *const c_void,
        );

        ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        // Lance le thread de recuperation des flux (hwnd passe en isize : Send).
        let hwnd_val = hwnd as isize;
        std::thread::spawn(move || worker(hwnd_val, cfg, pending));

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe fn state_of(hwnd: HWND) -> *mut State {
    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // WM_CREATE arrive pendant CreateWindowExW : on attache le pointeur d'etat.
    const WM_CREATE: u32 = 0x0001;
    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            let state_ptr = (*cs).lpCreateParams as *mut State;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            appbar::register(hwnd, &mut *state_ptr);
            SetTimer(hwnd, TIMER_ID, 16, None); // ~60 images/seconde
            0
        }
        WM_TIMER => {
            let st_ptr = state_of(hwnd);
            if !st_ptr.is_null() {
                let st = &mut *st_ptr;
                st.offset -= (st.speed * st.scale).max(1.0);
                // Boucle sans couture : un cycle entier a defile -> on revient d'une
                // periode (saut invisible, le bandeau se repete a l'identique).
                let period = st.strip.period as f64;
                if st.offset <= -period {
                    st.offset += period;
                }
                InvalidateRect(hwnd, ptr::null(), 0);
            }
            0
        }
        WM_APP_REFRESH => {
            // Nouveau lot de titres : on reconstruit le bandeau (meme police -> meme
            // hauteur, donc pas besoin de re-reserver l'espace AppBar).
            let st_ptr = state_of(hwnd);
            if !st_ptr.is_null() {
                let st = &mut *st_ptr;
                let items = st.pending.lock().unwrap().take();
                if let Some(items) = items {
                    st.strip = Strip::build(&st.cfg, st.dpi, st.scale, st.screen_w, &items);
                    // Conserve la position de defilement -> pas de saut "au debut" au refresh.
                    st.offset %= st.strip.period as f64;
                    InvalidateRect(hwnd, ptr::null(), 0);
                }
            }
            0
        }
        WM_PAINT => {
            paint(hwnd);
            0
        }
        WM_LBUTTONUP => {
            // Clic gauche : ouvrir l'article sous le curseur, s'il y en a un.
            let st_ptr = state_of(hwnd);
            if !st_ptr.is_null() {
                let st = &*st_ptr;
                let cx = (lparam & 0xffff) as i16 as i32; // LOWORD signe = x client
                let period = st.strip.period;
                let src_x = (-st.offset) as i32;
                let px = ((src_x + cx) % period + period) % period;
                if let Some(url) = st.strip.hit_test(px) {
                    if !url.is_empty() {
                        open_url(url);
                    }
                }
            }
            0
        }
        WM_RBUTTONUP => {
            show_menu(hwnd);
            0
        }
        WM_DESTROY => {
            let st_ptr = state_of(hwnd);
            if !st_ptr.is_null() {
                KillTimer(hwnd, TIMER_ID);
                appbar::unregister(&mut *st_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                drop(Box::from_raw(st_ptr)); // Strip::drop libere les objets GDI
            }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// Rendu d'une image : un seul BitBlt depuis le bandeau, a la position de
// defilement. offset dans (-period, 0] -> source = -offset, dans [0, period).
unsafe fn paint(hwnd: HWND) {
    let st_ptr = state_of(hwnd);
    let mut ps: PAINTSTRUCT = std::mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    if !st_ptr.is_null() {
        let st = &*st_ptr;
        let mut rc: RECT = std::mem::zeroed();
        GetClientRect(hwnd, &mut rc);
        let w = rc.right - rc.left;
        st.strip.blit(hdc, (-st.offset) as i32, w);
    }
    EndPaint(hwnd, &ps);
}

// Ouvre une URL dans le navigateur par defaut.
unsafe fn open_url(url: &str) {
    let verb = wide("open");
    let target = wide(url);
    ShellExecuteW(
        ptr::null_mut(),
        verb.as_ptr(),
        target.as_ptr(),
        ptr::null(),
        ptr::null(),
        SW_SHOWNORMAL,
    );
}

// Menu contextuel "Fermer la barre" (clic droit).
unsafe fn show_menu(hwnd: HWND) {
    let menu = CreatePopupMenu();
    let label = wide("Fermer la barre");
    AppendMenuW(menu, MF_STRING, MENU_CLOSE_ID, label.as_ptr());
    let mut pt: POINT = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_RIGHTBUTTON | TPM_RETURNCMD,
        pt.x,
        pt.y,
        0,
        hwnd,
        ptr::null(),
    );
    DestroyMenu(menu);
    if cmd as usize == MENU_CLOSE_ID {
        DestroyWindow(hwnd);
    }
}
