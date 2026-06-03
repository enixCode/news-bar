// Rendu : les titres RSS sont "cuits" UNE SEULE FOIS dans un bandeau bitmap
// (Strip), large d'un ecran + une periode. Chaque image = 1 seul BitBlt (leger,
// fluide). On memorise aussi la zone horizontale (hit-box) de chaque titre dans
// une periode, pour savoir quel article ouvrir au clic.

use std::ptr;

use crate::config::Config;
use crate::feeds::Item;

use windows_sys::Win32::Foundation::{RECT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, FillRect, GetDC, GetTextExtentPoint32W, GetTextMetricsW, ReleaseDC, SelectObject,
    SetBkMode, SetTextColor, TextOutW, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_PITCH, FF_DONTCARE, FW_BOLD, FW_NORMAL, HBITMAP, HDC, HGDIOBJ, OUT_DEFAULT_PRECIS,
    SRCCOPY, TEXTMETRICW, TRANSPARENT,
};

use super::{rgb, wide};

// Zone cliquable d'un titre dans une periode -> URL de l'article.
struct Hit {
    start: i32,
    end: i32,
    url: String,
}

pub(super) struct Strip {
    dc: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    pub(super) period: i32, // largeur d'un cycle complet (= avance du defilement)
    pub(super) bar_height: i32,
    hits: Vec<Hit>,
}

unsafe fn measure(dc: HDC, s16: &[u16]) -> i32 {
    let mut size = SIZE { cx: 0, cy: 0 };
    GetTextExtentPoint32W(dc, s16.as_ptr(), s16.len() as i32, &mut size);
    size.cx
}

impl Strip {
    pub(super) unsafe fn build(
        cfg: &Config,
        dpi: i32,
        scale: f64,
        screen_w: i32,
        items: &[Item],
    ) -> Strip {
        // Placeholder tant qu'aucun flux n'est arrive.
        let owned;
        let items: &[Item] = if items.is_empty() {
            owned = vec![Item {
                text: "news-bar : chargement des flux...".into(),
                url: String::new(),
            }];
            &owned
        } else {
            items
        };

        // Police. Taille config en POINTS -> px = taille * dpi / 72 (comme WinForms).
        let weight = if cfg.bold { FW_BOLD } else { FW_NORMAL } as i32;
        let font_px = -((cfg.font_size * dpi as f64 / 72.0).round() as i32);
        let face = wide(&cfg.font_name);
        let hfont = CreateFontW(
            font_px,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            (DEFAULT_PITCH as u32) | (FF_DONTCARE as u32),
            face.as_ptr(),
        );

        let screen_dc = GetDC(ptr::null_mut());

        // --- Mesures (police selectionnee dans le DC ecran) ---
        let restore = SelectObject(screen_dc, hfont as HGDIOBJ);
        let mut tm: TEXTMETRICW = std::mem::zeroed();
        GetTextMetricsW(screen_dc, &mut tm);
        let text_px_h = tm.tmHeight;

        let sep16 = wide(&cfg.separator);
        let sep_w = measure(screen_dc, &sep16);

        let mut glyphs: Vec<(Vec<u16>, i32, String)> = Vec::with_capacity(items.len());
        let mut period = 0;
        for it in items {
            let t16: Vec<u16> = it.text.encode_utf16().collect();
            let w = measure(screen_dc, &t16);
            period += w + sep_w;
            glyphs.push((t16, w, it.url.clone()));
        }
        let period = period.max(1);
        SelectObject(screen_dc, restore); // libere hfont du DC ecran

        let bar_height = text_px_h + (cfg.padding as f64 * scale) as i32;
        let text_y = (bar_height - text_px_h) / 2;

        // --- Bandeau (ecran + une periode : une tranche large d'une fenetre est
        // toujours pleine quel que soit l'offset). ---
        let strip_w = screen_w + period;
        let dc = CreateCompatibleDC(screen_dc);
        let bmp = CreateCompatibleBitmap(screen_dc, strip_w, bar_height);
        let old = SelectObject(dc, bmp as HGDIOBJ);

        let full = RECT { left: 0, top: 0, right: strip_w, bottom: bar_height };
        let bg_brush = CreateSolidBrush(rgb(cfg.bg));
        FillRect(dc, &full, bg_brush);
        DeleteObject(bg_brush as HGDIOBJ);

        let memfont = SelectObject(dc, hfont as HGDIOBJ);
        SetBkMode(dc, TRANSPARENT as i32);
        SetTextColor(dc, rgb(cfg.fg));

        let mut hits = Vec::new();
        let mut x = 0;
        let mut i = 0;
        while x < strip_w {
            let (ref t16, w, ref url) = glyphs[i % glyphs.len()];
            TextOutW(dc, x, text_y, t16.as_ptr(), t16.len() as i32);
            if x < period && !url.is_empty() {
                hits.push(Hit {
                    start: x,
                    end: x + w,
                    url: url.clone(),
                });
            }
            x += w + sep_w;
            i += 1;
        }

        SelectObject(dc, memfont); // libere hfont du DC memoire
        ReleaseDC(ptr::null_mut(), screen_dc);
        DeleteObject(hfont as HGDIOBJ);

        Strip {
            dc,
            bmp,
            old,
            period,
            bar_height,
            hits,
        }
    }

    // Copie une tranche large de `width` depuis la source `x_src`. Operation
    // unique couvrant tout le client : aucune dechirure.
    pub(super) unsafe fn blit(&self, hdc: HDC, x_src: i32, width: i32) {
        BitBlt(hdc, 0, 0, width, self.bar_height, self.dc, x_src, 0, SRCCOPY);
    }

    // period_x dans [0, period) -> URL de l'article a cet endroit (clic sur titre).
    pub(super) fn hit_test(&self, period_x: i32) -> Option<&str> {
        for h in &self.hits {
            if period_x >= h.start && period_x < h.end {
                return Some(&h.url);
            }
        }
        None
    }
}

impl Drop for Strip {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(self.bmp as HGDIOBJ);
            DeleteDC(self.dc);
        }
    }
}
