use bevy::prelude::*;
use bevy_simple_scroll_view::*;

const CLR_1: Color = Color::srgb(0.168, 0.168, 0.168);
const CLR_2: Color = Color::srgb(0.109, 0.109, 0.109);
const CLR_3: Color = Color::srgb(0.569, 0.592, 0.647);
const CLR_4: Color = Color::srgb(0.902, 0.4, 0.004);
const CLR_5: Color = Color::srgb(0.2, 0.3, 0.4);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ScrollViewPlugin))
        .add_systems(Startup, prepare)
        .add_systems(Update, reset_scroll)
        .run();
}

fn prepare(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            background_color: CLR_1.into(),
            ..default()
        })
        .with_children(|p| {
            // Reset button
            p.spawn(ButtonBundle {
                style: Style {
                    margin: UiRect::all(Val::Px(15.0)),
                    padding: UiRect::all(Val::Px(15.0)),
                    max_height: Val::Px(100.0),
                    border: UiRect::all(Val::Px(3.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: CLR_2.into(),
                border_color: CLR_4.into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "Reset scroll",
                    TextStyle {
                        font_size: 25.0,
                        color: CLR_4,
                        ..default()
                    },
                ));
            });
            // Main scroll view
            p.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(80.0),
                        height: Val::Percent(80.0),
                        margin: UiRect::all(Val::Px(15.0)),
                        ..default()
                    },
                    background_color: CLR_2.into(),
                    ..default()
                },
                ScrollView::default(),
            ))
            .with_children(|p| {
                p.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        ..default()
                    },
                    ScrollableContent::default(),
                ))
                .with_children(|scroll_area| {
                    // Add a nested scroll view
                    scroll_area
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    height: Val::Px(200.0),
                                    width: Val::Percent(100.0),
                                    margin: UiRect::all(Val::Px(15.0)),
                                    ..default()
                                },
                                background_color: CLR_5.into(),
                                ..default()
                            },
                            ScrollView::default(),
                        ))
                        .with_children(|p| {
                            p.spawn((
                                NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ScrollableContent::default(),
                            ))
                            .with_children(|inner_scroll| {
                                // Add content to nested scroll view
                                for i in 0..10 {
                                    inner_scroll
                                        .spawn(NodeBundle {
                                            style: Style {
                                                width: Val::Percent(95.0),
                                                margin: UiRect::all(Val::Px(10.0)),
                                                border: UiRect::all(Val::Px(3.0)),
                                                padding: UiRect::all(Val::Px(20.0)),
                                                ..default()
                                            },
                                            background_color: CLR_2.into(),
                                            border_color: CLR_4.into(),
                                            ..default()
                                        })
                                        .with_children(|p| {
                                            p.spawn(
                                                TextBundle::from_section(
                                                    format!("Inner {}", i),
                                                    TextStyle {
                                                        font_size: 20.0,
                                                        color: CLR_4,
                                                        ..default()
                                                    },
                                                )
                                                .with_text_justify(JustifyText::Center),
                                            );
                                        });
                                }
                            });
                        });

                    // Main scroll view content
                    for i in 0..21 {
                        scroll_area
                            .spawn(NodeBundle {
                                style: Style {
                                    min_width: Val::Px(200.0),
                                    margin: UiRect::all(Val::Px(15.0)),
                                    border: UiRect::all(Val::Px(5.0)),
                                    padding: UiRect::all(Val::Px(30.0)),
                                    ..default()
                                },
                                border_color: CLR_3.into(),
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn(
                                    TextBundle::from_section(
                                        format!("Outer {}", i),
                                        TextStyle {
                                            font_size: 25.0,
                                            color: CLR_3,
                                            ..default()
                                        },
                                    )
                                    .with_text_justify(JustifyText::Center),
                                );
                            });
                    }
                });
            });
        });
}

fn reset_scroll(
    q: Query<(&Button, &Interaction), Changed<Interaction>>,
    mut scrolls_q: Query<&mut ScrollableContent>,
) {
    for (_, interaction) in q.iter() {
        if interaction == &Interaction::Pressed {
            for mut scroll in scrolls_q.iter_mut() {
                scroll.pos_y = 0.0;
            }
        }
    }
}
