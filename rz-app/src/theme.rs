//! 设计系统：配色 / 字体 / 面板与文本绘制原语。

use egui::{
    Align, Align2, Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Pos2, Rect,
    Response, Rounding, Sense, Shape, Stroke, TextStyle, Ui, Vec2,
};

// ---------- 配色 ----------
pub const BG: Color32 = Color32::from_rgb(0x07, 0x09, 0x0A);
pub const SIDEBAR: Color32 = Color32::from_rgb(0x0A, 0x0D, 0x0B);
pub const CARD: Color32 = Color32::from_rgb(0x10, 0x15, 0x13);
pub const CARD_HI: Color32 = Color32::from_rgb(0x16, 0x1D, 0x19);
pub const TRACK: Color32 = Color32::from_rgb(0x1E, 0x27, 0x22);
pub const BORDER: Color32 = Color32::from_rgb(0x1D, 0x26, 0x21);
pub const BORDER_HI: Color32 = Color32::from_rgb(0x2E, 0x3F, 0x35);
pub const TEXT: Color32 = Color32::from_rgb(0xE9, 0xF2, 0xEC);
pub const DIM: Color32 = Color32::from_rgb(0x8B, 0x9B, 0x91);
pub const MUTED: Color32 = Color32::from_rgb(0x51, 0x60, 0x57);
pub const ACCENT: Color32 = Color32::from_rgb(0x44, 0xD6, 0x2C);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x0B, 0x1A, 0x0C);
pub const WARN: Color32 = Color32::from_rgb(0xFF, 0xB0, 0x2E);
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x5A, 0x5A);
pub const INFO: Color32 = Color32::from_rgb(0x5A, 0xC8, 0xFA);

pub fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

// ---------- 字体 ----------
#[derive(Clone)]
pub struct Type {
    pub title: FontId,
    pub h1: FontId,
    pub h2: FontId,
    pub body: FontId,
    pub small: FontId,
    pub tiny: FontId,
    pub mono: FontId,
    pub num: FontId,
    pub big: FontId,
    pub semi: FontFamily,
}

fn read_font(names: &[&str]) -> Option<Vec<u8>> {
    for n in names {
        let p = format!("C:\\Windows\\Fonts\\{}", n);
        if let Ok(b) = std::fs::read(&p) {
            if b.len() > 1024 {
                return Some(b);
            }
        }
    }
    None
}

pub fn install(ctx: &Context) -> Type {
    let mut fonts = FontDefinitions::default();
    let semi_family = FontFamily::Name("semi".into());

    if let Some(bytes) = read_font(&["segoeui.ttf"]) {
        fonts.font_data.insert("body".into(), FontData::from_owned(bytes).into());
    }
    if let Some(bytes) = read_font(&["seguisb.ttf", "segoeuib.ttf"]) {
        fonts.font_data.insert("semi".into(), FontData::from_owned(bytes).into());
    }
    if let Some(bytes) = read_font(&["Deng.ttf", "simhei.ttf"]) {
        fonts.font_data.insert("cjk".into(), FontData::from_owned(bytes).into());
    }
    if let Some(bytes) = read_font(&["consola.ttf"]) {
        fonts.font_data.insert("mono".into(), FontData::from_owned(bytes).into());
    }

    let body_family = if fonts.font_data.contains_key("body") {
        "body".to_string()
    } else {
        fonts
            .families
            .get(&FontFamily::Proportional)
            .and_then(|v| v.first().cloned())
            .unwrap_or_default()
    };
    let semibold_family = if fonts.font_data.contains_key("semi") {
        "semi".to_string()
    } else {
        body_family.clone()
    };

    let mut prop = Vec::new();
    if !body_family.is_empty() {
        prop.push(body_family.clone());
    }
    if fonts.font_data.contains_key("cjk") {
        prop.push("cjk".to_string());
    }
    for f in fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default()
    {
        if !prop.contains(&f) {
            prop.push(f);
        }
    }
    fonts.families.insert(FontFamily::Proportional, prop.clone());

    let mut mono = Vec::new();
    if fonts.font_data.contains_key("mono") {
        mono.push("mono".to_string());
    }
    if fonts.font_data.contains_key("cjk") {
        mono.push("cjk".to_string());
    }
    for f in fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default()
    {
        if !mono.contains(&f) {
            mono.push(f);
        }
    }
    fonts.families.insert(FontFamily::Monospace, mono.clone());

    let mut semi_list = vec![semibold_family];
    if fonts.font_data.contains_key("cjk") {
        semi_list.push("cjk".to_string());
    }
    fonts.families.insert(semi_family.clone(), semi_list);

    ctx.set_fonts(fonts);

    let ty = Type {
        title: FontId::new(23.0, semi_family.clone()),
        h1: FontId::new(17.0, semi_family.clone()),
        h2: FontId::new(14.5, semi_family.clone()),
        body: FontId::new(13.5, FontFamily::Proportional),
        small: FontId::new(12.0, FontFamily::Proportional),
        tiny: FontId::new(10.5, FontFamily::Proportional),
        mono: FontId::new(12.0, FontFamily::Monospace),
        num: FontId::new(30.0, semi_family.clone()),
        big: FontId::new(46.0, semi_family.clone()),
        semi: semi_family,
    };

    install_style(ctx, &ty);
    ty
}

fn install_style(ctx: &Context, ty: &Type) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    let v = &mut style.visuals;
    v.panel_fill = BG;
    v.window_fill = CARD;
    v.extreme_bg_color = Color32::from_rgb(0x0A, 0x0E, 0x0C);
    v.faint_bg_color = CARD_HI;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = alpha(ACCENT, 70);
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = ACCENT;
    v.widgets.noninteractive.bg_fill = CARD;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DIM);
    v.widgets.noninteractive.rounding = Rounding::same(10.0);
    v.widgets.inactive.bg_fill = CARD_HI;
    v.widgets.inactive.weak_bg_fill = CARD_HI;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.rounding = Rounding::same(9.0);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x1B, 0x24, 0x1F);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x1B, 0x24, 0x1F);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_HI);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.rounding = Rounding::same(9.0);
    v.widgets.active.bg_fill = alpha(ACCENT, 40);
    v.widgets.active.weak_bg_fill = alpha(ACCENT, 40);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.open.bg_fill = CARD_HI;
    v.widgets.open.bg_stroke = Stroke::new(1.0, BORDER_HI);

    style.spacing.item_spacing = Vec2::new(9.0, 9.0);
    style.spacing.button_padding = Vec2::new(13.0, 6.0);
    style.spacing.scroll.bar_width = 7.0;
    style.spacing.scroll.floating = true;
    style.spacing.interact_size.y = 24.0;

    style.text_styles = [
        (TextStyle::Heading, ty.h1.clone()),
        (TextStyle::Body, ty.body.clone()),
        (TextStyle::Button, ty.body.clone()),
        (TextStyle::Small, ty.small.clone()),
        (TextStyle::Monospace, ty.mono.clone()),
    ]
    .into();

    ctx.set_style(style);
}

// ---------- 绘制原语 ----------

/// 卡片容器：先占位、内容绘制完成后再回填背景（保持内容在背景之上）。
pub struct CardStyle {
    pub fill: Color32,
    pub stroke: Stroke,
    pub radius: f32,
    pub pad: f32,
}

impl CardStyle {
    pub fn new() -> Self {
        CardStyle {
            fill: CARD,
            stroke: Stroke::new(1.0, BORDER),
            radius: 14.0,
            pad: 18.0,
        }
    }
    pub fn flat(mut self) -> Self {
        self.stroke = Stroke::NONE;
        self
    }
    pub fn pad(mut self, p: f32) -> Self {
        self.pad = p;
        self
    }
    pub fn fill(mut self, c: Color32) -> Self {
        self.fill = c;
        self
    }
}

pub fn card<R>(ui: &mut Ui, st: CardStyle, add: impl FnOnce(&mut Ui) -> R) -> (Rect, R) {
    let shadow_idx = ui.painter().add(Shape::Noop);
    let idx = ui.painter().add(Shape::Noop);
    let avail = ui.available_width();
    let pad = st.pad;
    let resp = ui.scope(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
        ui.add_space(pad);
        let inner = ui
            .horizontal(|ui| {
                ui.add_space(pad);
                let r = ui
                    .vertical(|ui| {
                        ui.set_max_width((avail - pad * 2.0).max(24.0));
                        add(ui)
                    })
                    .inner;
                ui.add_space(pad);
                r
            })
            .inner;
        ui.add_space(pad);
        inner
    });
    let rect = Rect::from_min_size(
        resp.response.rect.min,
        Vec2::new(avail, resp.response.rect.height()),
    );
    let shadow = Shape::Vec(vec![
        Shape::rect_filled(
            rect.translate(Vec2::new(0.0, 9.0)).expand(7.0),
            st.radius + 7.0,
            Color32::from_black_alpha(22),
        ),
        Shape::rect_filled(
            rect.translate(Vec2::new(0.0, 3.0)).expand(2.0),
            st.radius + 2.0,
            Color32::from_black_alpha(26),
        ),
    ]);
    ui.painter().set(shadow_idx, shadow);
    ui.painter().set(idx, Shape::rect_filled(rect, st.radius, st.fill));
    if st.stroke.width > 0.0 {
        ui.painter().rect_stroke(rect, st.radius, st.stroke);
        // 顶部高光，模拟玻璃质感
        let hl = Rect::from_min_size(
            rect.min + Vec2::new(st.radius, 0.0),
            Vec2::new(rect.width() - st.radius * 2.0, 1.0),
        );
        ui.painter()
            .rect_filled(hl, 0.0, alpha(Color32::WHITE, 12));
    }
    (rect, resp.inner)
}

pub fn section_title(ui: &mut Ui, ty: &Type, title: &str, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title).font(ty.h2.clone()).color(TEXT));
        if !hint.is_empty() {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(hint).font(ty.small.clone()).color(MUTED));
        }
    });
}

pub fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, alpha(BORDER, 200));
}

pub fn key_value(ui: &mut Ui, ty: &Type, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(key).font(ty.small.clone()).color(MUTED));
        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
            ui.label(egui::RichText::new(value).font(ty.small.clone()).color(DIM));
        });
    });
}

/// 圆角标签
pub fn pill(ui: &mut Ui, ty: &Type, text: &str, color: Color32) -> Response {
    let galley = ui.painter().layout_no_wrap(
        text.to_string(),
        ty.tiny.clone(),
        color,
    );
    let size = Vec2::new(galley.size().x + 16.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(10.0), alpha(color, 26));
    ui.painter().rect_stroke(
        rect,
        Rounding::same(10.0),
        Stroke::new(1.0, alpha(color, 60)),
    );
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        color,
    );
    resp
}

/// 圆弧点集
pub fn arc_points(center: Pos2, radius: f32, start: f32, end: f32, segments: usize) -> Vec<Pos2> {
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let a = start + (end - start) * t;
            center + Vec2::new(a.cos(), a.sin()) * radius
        })
        .collect()
}

/// 单行文本（带锚点）
#[allow(clippy::too_many_arguments)]
pub fn label_at(
    painter: &egui::Painter,
    pos: Pos2,
    anchor: Align2,
    text: String,
    font: FontId,
    color: Color32,
) {
    painter.text(pos, anchor, text, font, color);
}

/// 柔和径向光斑：多层低透明度叠加，避免同心圆硬边。
pub fn soft_glow(
    painter: &egui::Painter,
    center: Pos2,
    radius: Vec2,
    color: Color32,
    layers: usize,
    strength: u8,
) {
    let n = layers.max(2) as f32;
    for i in 0..layers {
        let t = 1.0 - i as f32 / n;
        let r = radius * (0.30 + 0.70 * t);
        painter.add(Shape::ellipse_filled(
            center,
            r,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), strength),
        ));
    }
}
