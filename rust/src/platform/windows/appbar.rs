// Reservation d'espace ecran via l'API AppBar (shell32) : le meme mecanisme que
// la barre des taches. Windows place la barre, recule la zone de travail des
// autres fenetres (maximisees comprises), et libere tout a la fermeture.
// Port direct de AppBar.ps1.

use super::State;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Shell::{
    SHAppBarMessage, ABE_BOTTOM, ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE};

pub(super) unsafe fn register(hwnd: HWND, st: &mut State) {
    let mut abd: APPBARDATA = std::mem::zeroed();
    abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
    abd.hWnd = hwnd;
    abd.uEdge = if st.is_bottom { ABE_BOTTOM } else { ABE_TOP };
    SHAppBarMessage(ABM_NEW, &mut abd);

    let h = st.strip.bar_height;
    abd.rc.left = 0;
    abd.rc.right = st.screen_w;
    if st.is_bottom {
        abd.rc.bottom = st.screen_h;
        abd.rc.top = st.screen_h - h;
    } else {
        abd.rc.top = 0;
        abd.rc.bottom = h;
    }
    SHAppBarMessage(ABM_QUERYPOS, &mut abd);

    // Recale la hauteur sous le bord approuve par Windows.
    if st.is_bottom {
        abd.rc.top = abd.rc.bottom - h;
    } else {
        abd.rc.bottom = abd.rc.top + h;
    }
    SHAppBarMessage(ABM_SETPOS, &mut abd);

    let r = abd.rc;
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        r.left,
        r.top,
        r.right - r.left,
        r.bottom - r.top,
        SWP_NOACTIVATE,
    );

    st.abd = abd;
    st.abd_set = true;
}

pub(super) unsafe fn unregister(st: &mut State) {
    if st.abd_set {
        SHAppBarMessage(ABM_REMOVE, &mut st.abd);
        st.abd_set = false;
    }
}
