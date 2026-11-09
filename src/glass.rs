use gpui_kit::component::ActiveTheme as _;
use gpui_kit::*;

use crate::tokens;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    Bar,
    Panel,
}

pub fn shadows(depth: Depth, dark: bool) -> Vec<BoxShadow> {
    let ink = black();
    let boost = if dark { 2. } else { 1. };
    let mut out = vec![
        BoxShadow::new(
            px(0.),
            tokens::HAIRLINE,
            ink.opacity(tokens::GLASS_CONTACT * boost),
        )
        .blur_radius(tokens::GLASS_CONTACT_BLUR),
        BoxShadow::new(
            px(0.),
            tokens::GLASS_AMBIENT_Y,
            ink.opacity(tokens::GLASS_AMBIENT * boost),
        )
        .blur_radius(tokens::GLASS_AMBIENT_BLUR),
    ];
    if depth == Depth::Panel {
        out.push(
            BoxShadow::new(
                px(0.),
                tokens::GLASS_LIFT_Y,
                ink.opacity(tokens::GLASS_LIFT * boost),
            )
            .blur_radius(tokens::GLASS_LIFT_BLUR),
        );
    }
    out.push(BoxShadow::new(px(0.), tokens::HAIRLINE, white().opacity(rim(dark))).inset());
    out.push(
        BoxShadow::new(
            px(0.),
            -tokens::HAIRLINE,
            ink.opacity(tokens::GLASS_SHADE * boost),
        )
        .inset(),
    );
    out
}

fn rim(dark: bool) -> f32 {
    if dark {
        tokens::GLASS_RIM_DARK
    } else {
        tokens::GLASS_RIM
    }
}

pub fn fill(cx: &App) -> Background {
    let theme = cx.theme();
    let dark = theme.is_dark();
    let base = theme.popover;
    let body = base.alpha(if dark {
        tokens::GLASS_FILL_DARK
    } else {
        tokens::GLASS_FILL
    });
    let lift = if dark {
        tokens::GLASS_SHEEN_DARK
    } else {
        tokens::GLASS_SHEEN
    };
    let sheen = body.blend(white().alpha(lift)).alpha(body.a);
    linear_gradient(
        tokens::GLASS_SHEEN_ANGLE,
        linear_color_stop(sheen, 0.),
        linear_color_stop(body, tokens::GLASS_SHEEN_STOP),
    )
}

pub fn surface<E: Styled>(element: E, depth: Depth, cx: &App) -> E {
    let theme = cx.theme();
    let dark = theme.is_dark();
    element
        .bg(fill(cx))
        .border_1()
        .border_color(theme.foreground.opacity(tokens::GLASS_EDGE))
        .shadow(shadows(depth, dark))
        .rounded(match depth {
            Depth::Bar => tokens::RADIUS_LG,
            Depth::Panel => tokens::RADIUS_XL,
        })
}
