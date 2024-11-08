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

fn input_mouse_pressed_move(
    mut motion_evr: EventReader<MouseMotion>,
    mut q: Query<(&Children, &Interaction, &ScrollView, &Node), With<ScrollView>>,
    mut content_q: Query<(&mut ScrollableContent, &Node)>,
) {
    for evt in motion_evr.read() {
        // Get all pressed scroll views
        let pressed_scrolls: Vec<_> = q
            .iter_mut()
            .filter(|(_, &interaction, _, _)| interaction == Interaction::Pressed)
            .collect();

        let mut consumed = false;

        // Process from innermost (last) to outermost (first)
        for (children, _, scroll_view, node) in pressed_scrolls.into_iter().rev() {
            if consumed {
                continue;
            }

            let container_height = node.size().y;
            let mut scroll_applied = false;

            for &child in children.iter() {
                if let Ok(item) = content_q.get_mut(child) {
                    let mut scroll = item.0;
                    let max_scroll = (item.1.size().y - container_height).max(0.0);

                    // Check if we're at the scroll boundaries
                    let new_pos = scroll.pos_y + evt.delta.y;
                    let will_clamp = new_pos < -max_scroll || new_pos > 0.;

                    scroll.pos_y += evt.delta.y;
                    scroll.pos_y = scroll.pos_y.clamp(-max_scroll, 0.);

                    // If we actually scrolled and didn't hit the boundaries, mark as applied
                    if !will_clamp && max_scroll > 0.0 {
                        scroll_applied = true;
                    }
                }
            }

            // If we applied scroll and don't want to propagate, mark as consumed
            if scroll_applied && !scroll_view.propagate {
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

        // Get all pressed scroll views
        let pressed_scrolls: Vec<_> = q
            .iter_mut()
            .filter(|(_, &interaction, _, _)| interaction == Interaction::Pressed)
            .collect();

        let mut consumed = false;

        // Process from innermost (last) to outermost (first)
        for (children, _, scroll_view, node) in pressed_scrolls.into_iter().rev() {
            if consumed {
                continue;
            }

            let container_height = node.size().y;
            let mut scroll_applied = false;

            for &child in children.iter() {
                if let Ok(item) = content_q.get_mut(child) {
                    let mut scroll = item.0;
                    let max_scroll = (item.1.size().y - container_height).max(0.0);

                    // Check if we're at the scroll boundaries
                    let new_pos = scroll.pos_y + touch.delta().y;
                    let will_clamp = new_pos < -max_scroll || new_pos > 0.;

                    scroll.pos_y += touch.delta().y;
                    scroll.pos_y = scroll.pos_y.clamp(-max_scroll, 0.);

                    // If we actually scrolled and didn't hit the boundaries, mark as applied
                    if !will_clamp && max_scroll > 0.0 {
                        scroll_applied = true;
                    }
                }
            }

            // If we applied scroll and don't want to propagate, mark as consumed
            if scroll_applied && !scroll_view.propagate {
                consumed = true;
            }
        }
    }
}

fn scroll_events(
    mut scroll_evr: EventReader<MouseWheel>,
    mut q: Query<(Entity, &Children, &Interaction, &ScrollView, &Node), With<ScrollView>>,
    time: Res<Time>,
    mut content_q: Query<(&mut ScrollableContent, &Node)>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.read() {
        // Get all hovered scroll views
        let hovered_scrolls: Vec<_> = q
            .iter_mut()
            .filter(|(_, _, &interaction, _, _)| interaction == Interaction::Hovered)
            .collect();

        // If we have multiple hovered scroll views, we only want to scroll the innermost one
        // unless propagation is enabled
        let mut consumed = false;

        // Process from innermost (last) to outermost (first)
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

            let container_height = node.size().y;
            let mut scroll_applied = false;

            for &child in children.iter() {
                if let Ok(item) = content_q.get_mut(child) {
                    let y = y * time.delta().as_secs_f32() * scroll_view.scroll_speed;
                    let mut scroll = item.0;
                    let max_scroll = (item.1.size().y - container_height).max(0.0);

                    // Check if we're at the scroll boundaries
                    let new_pos = scroll.pos_y + y;
                    let will_clamp = new_pos < -max_scroll || new_pos > 0.;

                    scroll.pos_y += y;
                    scroll.pos_y = scroll.pos_y.clamp(-max_scroll, 0.);

                    // If we actually scrolled and didn't hit the boundaries, mark as applied
                    if !will_clamp && max_scroll > 0.0 {
                        scroll_applied = true;
                    }
                }
            }

            // If we applied scroll and don't want to propagate, mark as consumed
            if scroll_applied && !scroll_view.propagate {
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
