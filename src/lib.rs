#![doc = include_str!("../README.md")]

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

/// A `Plugin` providing the systems and components required to make a ScrollView work.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_simple_scroll_view::*;
///
/// App::new()
///     .add_plugins((DefaultPlugins,ScrollViewPlugin))
///     .run();
/// ```
pub struct ScrollViewPlugin;

impl Plugin for ScrollViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScrollView>()
            .register_type::<ScrollableContent>()
            .add_systems(
                Update,
                (
                    create_scroll_view,
                    input_mouse_pressed_move,
                    input_touch_pressed_move,
                    scroll_events,
                    scroll_update,
                )
                    .chain(),
            );
    }
}

/// Root component of scroll, it should have clipped style.
#[derive(Component, Debug, Reflect)]
pub struct ScrollView {
    /// Field which control speed of the scrolling.
    /// Could be negative number to implement invert scroll
    pub scroll_speed: f32,
    /// Controls whether scroll events should propagate to parent scroll views
    /// Default is false.
    pub propagate: bool,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self {
            scroll_speed: 200.0,
            propagate: false,
        }
    }
}

/// Component containing offset value of the scroll container to the parent.
/// It is possible to update the field `pos_y` manually to move scrollview to desired location.
#[derive(Component, Debug, Reflect, Default)]
pub struct ScrollableContent {
    /// Scroll container offset to the `ScrollView`.
    pub pos_y: f32,
}

pub fn create_scroll_view(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Style), Added<ScrollView>>,
) {
    for (e, mut style) in q.iter_mut() {
        style.overflow = Overflow::clip();
        style.align_items = AlignItems::Start;
        style.align_self = AlignSelf::Stretch;
        style.flex_direction = FlexDirection::Row;
        commands.entity(e).insert(Interaction::None);
    }
}

// Common helper function to handle scroll logic
fn handle_scroll_for_view(
    children: &Children,
    scroll_view: &ScrollView,
    node: &Node,
    delta: f32,
    content_q: &mut Query<(&mut ScrollableContent, &Node)>,
) -> (bool, bool) {
    let container_height = node.size().y;
    let mut scroll_applied = false;
    let mut at_boundary = false;

    for &child in children.iter() {
        if let Ok(item) = content_q.get_mut(child) {
            let mut scroll = item.0;
            let max_scroll = (item.1.size().y - container_height).max(0.0);
            
            // Check if we're at the scroll boundaries
            let new_pos = scroll.pos_y + delta;
            let will_hit_top = new_pos > 0.;
            let will_hit_bottom = new_pos < -max_scroll;
            
            scroll.pos_y += delta;
            scroll.pos_y = scroll.pos_y.clamp(-max_scroll, 0.);
            
            // If we have scrollable content
            if max_scroll > 0.0 {
                if !will_hit_top && !will_hit_bottom {
                    scroll_applied = true;
                } else {
                    at_boundary = true;
                }
            }
        }
    }

    // Consume the event if:
    // 1. We actually scrolled and propagation is disabled, OR
    // 2. We hit a boundary and propagation is disabled
    let should_consume = !scroll_view.propagate && (scroll_applied || at_boundary);
    (should_consume, at_boundary)
}

fn scroll_events(
    mut scroll_evr: EventReader<MouseWheel>,
    mut q: Query<(Entity, &Children, &Interaction, &ScrollView, &Node), With<ScrollView>>,
    time: Res<Time>,
    mut content_q: Query<(&mut ScrollableContent, &Node)>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.read() {
        let hovered_scrolls: Vec<_> = q
            .iter_mut()
            .filter(|(_, _, &interaction, _, _)| interaction == Interaction::Hovered)
            .collect();

        let mut consumed = false;
        
        for (_entity, children, _, scroll_view, node) in hovered_scrolls.into_iter().rev() {
            if consumed {
                continue;
            }

            let y = match ev.unit {
                MouseScrollUnit::Line => {
                    ev.y * time.delta().as_secs_f32() * scroll_view.scroll_speed
                }
                MouseScrollUnit::Pixel => ev.y,
            };
            
            let delta = y * time.delta().as_secs_f32() * scroll_view.scroll_speed;
            let (should_consume, _) = handle_scroll_for_view(children, scroll_view, node, delta, &mut content_q);
            
            if should_consume {
                consumed = true;
            }
        }
    }
}

fn input_mouse_pressed_move(
    mut motion_evr: EventReader<MouseMotion>,
    mut q: Query<(&Children, &Interaction, &ScrollView, &Node), With<ScrollView>>,
    mut content_q: Query<(&mut ScrollableContent, &Node)>,
) {
    for evt in motion_evr.read() {
        let pressed_scrolls: Vec<_> = q
            .iter_mut()
            .filter(|(_, &interaction, _, _)| interaction == Interaction::Pressed)
            .collect();

        let mut consumed = false;

        for (children, _, scroll_view, node) in pressed_scrolls.into_iter().rev() {
            if consumed {
                continue;
            }

            let (should_consume, _) = handle_scroll_for_view(children, scroll_view, node, evt.delta.y, &mut content_q);
            
            if should_consume {
                consumed = true;
            }
        }
    }
}

fn input_touch_pressed_move(
    touches: Res<Touches>,
    mut q: Query<(&Children, &Interaction, &ScrollView, &Node), With<ScrollView>>,
    mut content_q: Query<(&mut ScrollableContent, &Node)>,
) {
    for t in touches.iter() {
        let Some(touch) = touches.get_pressed(t.id()) else {
            continue;
        };

        let pressed_scrolls: Vec<_> = q
            .iter_mut()
            .filter(|(_, &interaction, _, _)| interaction == Interaction::Pressed)
            .collect();

        let mut consumed = false;

        for (children, _, scroll_view, node) in pressed_scrolls.into_iter().rev() {
            if consumed {
                continue;
            }

            let (should_consume, _) = handle_scroll_for_view(children, scroll_view, node, touch.delta().y, &mut content_q);
            
            if should_consume {
                consumed = true;
            }
        }
    }
}

fn scroll_update(mut q: Query<(&ScrollableContent, &mut Style), Changed<ScrollableContent>>) {
    for (scroll, mut style) in q.iter_mut() {
        style.top = Val::Px(scroll.pos_y);
    }
}
