//! 鼠标矢量插画（纯 egui 路径绘制，无位图资源）与品牌标识。

use crate::theme::*;
use egui::{Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2};

pub const BTN_LEFT: usize = 0;
pub const BTN_RIGHT: usize = 1;
pub const BTN_MIDDLE: usize = 2;
pub const BTN_BACK: usize = 3;
pub const BTN_FORWARD: usize = 4;

fn cubic(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, steps: usize) -> Vec<Vec2> {
    (1..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            let mt = 1.0 - t;
            p0 * (mt * mt * mt)
                + p1 * (3.0 * mt * mt * t)
                + p2 * (3.0 * mt * t * t)
                + p3 * (t * t * t)
        })
        .collect()
}

/// 归一化轮廓（0..1），沿外圈顺时针采样。
fn outline() -> Vec<Vec2> {
    let v = |x: f32, y: f32| Vec2::new(x, y);
    let mut pts: Vec<Vec2> = Vec::new();
    pts.push(v(0.50, 0.030));
    // 右上 -> 右侧 -> 右下
    pts.extend(cubic(v(0.50, 0.02), v(0.660, 0.030), v(0.845, 0.110), v(0.918, 0.290), 9));
    pts.extend(cubic(v(0.918, 0.290), v(0.972, 0.430), v(0.930, 0.690), v(0.812, 0.855), 9));
    pts.extend(cubic(v(0.812, 0.855), v(0.712, 0.975), v(0.288, 0.975), v(0.188, 0.855), 9));
    pts.extend(cubic(v(0.188, 0.855), v(0.070, 0.690), v(0.028, 0.430), v(0.082, 0.290), 9));
    pts.extend(cubic(v(0.082, 0.290), v(0.155, 0.110), v(0.340, 0.030), v(0.500, 0.020), 9));
    pts
}

fn to_screen(area: Rect, p: Vec2) -> Pos2 {
    Pos2::new(area.left() + p.x * area.width(), area.top() + p.y * area.height())
}

/// 绘制鼠标；返回 5 个按键热区（选中/悬停的按键会高亮）。
pub fn draw_mouse(
    painter: &Painter,
    area: Rect,
    accent: Color32,
    selected: Option<usize>,
    hover: Option<usize>,
) -> [Rect; 5] {
    // 保持 0.74 的宽高比，避免被拉伸成胶囊形
    let area = Rect::from_center_size(
        area.center(),
        Vec2::new(area.width().min(area.height() * 0.74), area.height()),
    );
    let p = painter.with_clip_rect(area.expand(26.0));
    let pts: Vec<Pos2> = outline().into_iter().map(|v| to_screen(area, v)).collect();


    // 外壳
    p.add(Shape::convex_polygon(
        pts.clone(),
        Color32::from_rgb(0x16, 0x1D, 0x19),
        Stroke::new(1.2, mix(BORDER_HI, TEXT, 0.18)),
    ));

    // 左右主键面
    let split_y = 0.44f32;
    let mut left: Vec<Pos2> = outline()
        .into_iter()
        .filter(|v| v.y <= split_y && v.x <= 0.52)
        .map(|v| to_screen(area, v))
        .collect();
    left.push(to_screen(area, Vec2::new(0.50, split_y)));
    left.push(to_screen(area, Vec2::new(0.10, split_y)));
    let mut right: Vec<Pos2> = outline()
        .into_iter()
        .filter(|v| v.y <= split_y && v.x >= 0.48)
        .map(|v| to_screen(area, v))
        .collect();
    right.push(to_screen(area, Vec2::new(0.50, split_y)));
    right.push(to_screen(area, Vec2::new(0.90, split_y)));

    let highlight = |which: usize| -> Option<Color32> {
        if selected == Some(which) {
            Some(alpha(accent, 60))
        } else if hover == Some(which) {
            Some(alpha(accent, 30))
        } else {
            None
        }
    };

    for (idx, shape_pts) in [(BTN_LEFT, &left), (BTN_RIGHT, &right)] {
        if let Some(c) = highlight(idx) {
            p.add(Shape::convex_polygon(shape_pts.clone(), c, Stroke::NONE));
        }
    }
    // 主键分缝
    p.line_segment(
        [
            to_screen(area, Vec2::new(0.5, 0.035)),
            to_screen(area, Vec2::new(0.5, split_y)),
        ],
        Stroke::new(1.0, alpha(Color32::from_rgb(0x33, 0x40, 0x38), 220)),
    );
    p.line_segment(
        [
            to_screen(area, Vec2::new(0.075, split_y)),
            to_screen(area, Vec2::new(0.925, split_y)),
        ],
        Stroke::new(1.0, alpha(Color32::from_rgb(0x2C, 0x38, 0x31), 200)),
    );

    // 滚轮
    let wheel = Rect::from_center_size(
        to_screen(area, Vec2::new(0.5, 0.185)),
        Vec2::new(area.width() * 0.085, area.height() * 0.20),
    );
    if let Some(c) = highlight(BTN_MIDDLE) {
        p.rect_filled(wheel.expand(5.0), Rounding::same(9.0), c);
    }
    p.rect_filled(wheel.expand2(Vec2::new(3.0, 5.0)), Rounding::same(9.0), mix(BG, TEXT, 0.04));
    p.rect_stroke(
        wheel.expand2(Vec2::new(3.0, 5.0)),
        Rounding::same(9.0),
        Stroke::new(1.0, alpha(BORDER_HI, 210)),
    );
    p.rect_filled(wheel, Rounding::same(6.0), Color32::from_rgb(0x0A, 0x0D, 0x0B));
    p.rect_stroke(wheel, Rounding::same(6.0), Stroke::new(1.2, mix(BORDER_HI, accent, 0.35)));
    // 滚轮纹理
    for i in 1..4 {
        let y = wheel.top() + wheel.height() * i as f32 / 4.0;
        p.line_segment(
            [Pos2::new(wheel.left() + 4.0, y), Pos2::new(wheel.right() - 4.0, y)],
            Stroke::new(1.0, alpha(accent, 90)),
        );
    }

    // 侧键
    let side_back = Rect::from_min_max(
        to_screen(area, Vec2::new(0.022, 0.520)),
        to_screen(area, Vec2::new(0.115, 0.612)),
    );
    let side_fwd = Rect::from_min_max(
        to_screen(area, Vec2::new(0.022, 0.632)),
        to_screen(area, Vec2::new(0.115, 0.724)),
    );
    for (idx, r) in [(BTN_BACK, side_back), (BTN_FORWARD, side_fwd)] {
        if let Some(c) = highlight(idx) {
            p.rect_filled(r.expand(4.0), Rounding::same(7.0), c);
        }
        p.rect_filled(r, Rounding::same(5.0), Color32::from_rgb(0x1B, 0x24, 0x1F));
        p.rect_stroke(r, Rounding::same(5.0), Stroke::new(1.0, BORDER_HI));
    }

    // 掌托品牌光带
    let brand = Rect::from_center_size(
        to_screen(area, Vec2::new(0.5, 0.80)),
        Vec2::new(area.width() * 0.13, area.height() * 0.022),
    );
    for k in [5.0, 3.0, 1.5, 0.0] {
        p.rect_filled(
            brand.expand(k),
            Rounding::same(4.0),
            alpha(accent, if k > 0.0 { 8 } else { 110 }),
        );
    }

    [
        Rect::from_min_max(
            to_screen(area, Vec2::new(0.09, 0.04)),
            to_screen(area, Vec2::new(0.485, split_y)),
        ),
        Rect::from_min_max(
            to_screen(area, Vec2::new(0.515, 0.04)),
            to_screen(area, Vec2::new(0.91, split_y)),
        ),
        wheel.expand(3.0),
        side_back.expand(4.0),
        side_fwd.expand(4.0),
    ]
}

/// 品牌标识：圆角方块 + 闪电
pub fn logo(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_filled(rect, Rounding::same(rect.width() * 0.26), alpha(color, 26));
    painter.rect_stroke(
        rect,
        Rounding::same(rect.width() * 0.26),
        Stroke::new(1.0, alpha(color, 90)),
    );
    let pts = vec![
        rect.center() + Vec2::new(rect.width() * 0.10, -rect.height() * 0.26),
        rect.center() + Vec2::new(-rect.width() * 0.22, rect.height() * 0.04),
        rect.center() + Vec2::new(-rect.width() * 0.01, rect.height() * 0.04),
        rect.center() + Vec2::new(-rect.width() * 0.08, rect.height() * 0.26),
        rect.center() + Vec2::new(rect.width() * 0.24, -rect.height() * 0.03),
        rect.center() + Vec2::new(rect.width() * 0.02, -rect.height() * 0.03),
    ];
    painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}
