//! Modal dialogs: playlist details, confirmations, shortcuts.

use egui::{Align, CornerRadius, Frame, Layout, Margin, Stroke};

use crate::app::App;
use crate::i18n::{Locale, gettext, ngettext};
use crate::model::{Action, Artwork, Dialog};
use crate::theme;

pub fn show(app: &mut App, ctx: &egui::Context) {
    let Some(dialog) = app.dialog.clone() else {
        return;
    };
    if let Dialog::Artwork(picture) = &dialog {
        artwork(app, ctx, picture);
        return;
    }
    let palette = app.palette;
    let locale = app.locale;
    let frame = Frame::new()
        .fill(palette.overlay)
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(theme::RADIUS + 4))
        .inner_margin(Margin::same(24))
        .shadow(egui::epaint::Shadow {
            offset: [0, 10],
            blur: 40,
            spread: 0,
            color: palette.shadow,
        });
    let response = egui::Modal::new(egui::Id::new("dialog"))
        .frame(frame)
        .backdrop_color(egui::Color32::from_black_alpha(if palette.dark {
            150
        } else {
            80
        }))
        .show(ctx, |ui| {
            ui.set_width(420.0);
            match dialog {
                Dialog::PersonalAppIntro => {
                    theme::text(ui, gettext(locale, "Spend less time waiting for Spotify"), theme::bold(20.0), palette.text);
                    ui.add_space(12.0);
                    for text in [
                        gettext(locale, "Spotifast's default connection shares Spotify's request limit with other listeners. When it gets busy, loading music and using playback controls can take longer."),
                        gettext(locale, "Your Premium account lets you create a free personal Spotify app. Connect it here to give supported requests your own allowance. Some pages still use the shared connection."),
                        gettext(locale, "Setup takes a few minutes. You can also find it later in Settings under Personal Spotify app."),
                    ] {
                        ui.add(egui::Label::new(egui::RichText::new(text).font(theme::regular(14.0)).color(palette.secondary)).wrap());
                        ui.add_space(10.0);
                    }
                    ui.add_space(8.0);
                    buttons(ui, |ui| {
                        if theme::pill_button(ui, &palette, &gettext(locale, "Set up personal app"), true).clicked() {
                            app.actions.push(Action::OpenPersonalAppSetup);
                        }
                        if theme::pill_button(ui, &palette, &gettext(locale, "Keep shared app"), false).clicked() {
                            app.actions.push(Action::CloseDialog);
                        }
                    });
                }
                // Drawn without the dialog's card, above.
                Dialog::Artwork(_) => {}
                Dialog::CreatePlaylist { .. } => create_playlist(app, ui),
                Dialog::EditPlaylist { .. } => edit_playlist(app, ui),
                Dialog::ConfirmDeletePlaylist { id, name, owned } => {
                    theme::text(
                        ui,
                        if owned {
                            gettext(locale, "Delete playlist?")
                        } else {
                            gettext(locale, "Remove from Your Library?")
                        },
                        theme::bold(20.0),
                        palette.text,
                    );
                    ui.add_space(8.0);
                    let body = if owned {
                        // Translators: {name} is a playlist name.
                        gettext(locale, "Delete “{name}”? You can recover it from Spotify for 90 days.")
                    } else {
                        // Translators: {name} is a playlist name.
                        gettext(locale, "“{name}” will no longer appear in Your Library.")
                    }
                    .replace("{name}", &name);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(body)
                                .font(theme::regular(14.0))
                                .color(palette.secondary),
                        )
                        .wrap(),
                    );
                    ui.add_space(20.0);
                    buttons(ui, |ui| {
                        if theme::pill_button(
                            ui,
                            &palette,
                            &if owned {
                                gettext(locale, "Delete")
                            } else {
                                gettext(locale, "Remove")
                            },
                            true,
                        )
                        .clicked()
                        {
                            app.actions.push(Action::DeletePlaylist(id.clone()));
                        }
                        if theme::pill_button(ui, &palette, &gettext(locale, "Cancel"), false).clicked() {
                            app.actions.push(Action::CloseDialog);
                        }
                    });
                }
                Dialog::ConfirmPlaylistDuplicates {
                    playlist_id,
                    playlist_name,
                    items,
                    position,
                    duplicate_uris,
                } => {
                    let multiple = items.len() > 1;
                    theme::text(
                        ui,
                        if multiple {
                            gettext(locale, "Songs already in this playlist")
                        } else {
                            gettext(locale, "Song already in this playlist")
                        },
                        theme::bold(20.0),
                        palette.text,
                    );
                    ui.add_space(8.0);
                    let body = duplicate_message(locale, &playlist_name, &items, &duplicate_uris);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(body)
                                .font(theme::regular(14.0))
                                .color(palette.secondary),
                        )
                        .wrap(),
                    );
                    ui.add_space(20.0);
                    buttons(ui, |ui| {
                        if theme::pill_button(ui, &palette, &gettext(locale, "Add anyway"), true).clicked() {
                            app.actions.push(Action::ConfirmAddToPlaylist {
                                playlist_id: playlist_id.clone(),
                                playlist_name: playlist_name.clone(),
                                items: items.clone(),
                                position,
                            });
                        }
                        if theme::pill_button(ui, &palette, &gettext(locale, "Cancel"), false).clicked() {
                            app.actions.push(Action::CloseDialog);
                        }
                    });
                }
                Dialog::Shortcuts => {
                    theme::text(ui, gettext(locale, "Keyboard shortcuts"), theme::bold(20.0), palette.text);
                    ui.add_space(12.0);
                    // `theme::text` truncates, which in a grid makes each cell
                    // claim almost no width and turns "Ctrl+Shift+A" into
                    // "Ctrl…". A shortcut is unusable when abbreviated, so
                    // these cells are sized to their content.
                    let cell = |ui: &mut egui::Ui, text: &str, font: egui::FontId, color| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(text).font(font).color(color))
                                .extend()
                                .selectable(false),
                        );
                    };
                    // The list is longer than a small screen is tall, so it
                    // scrolls inside the dialog rather than running off the
                    // bottom with the Done button beyond reach.
                    let room = ui.ctx().content_rect().height() - 190.0;
                    crate::autoscroll::show(
                        ui,
                        egui::ScrollArea::vertical()
                        .max_height(room.max(120.0))
                        .auto_shrink([false, true]),
                        egui::Vec2b::new(false, true),
                        |ui| {
                            egui::Grid::new("shortcuts")
                                .num_columns(2)
                                .spacing([24.0, 8.0])
                                .show(ui, |ui| {
                                    for (keys, description) in super::keys::shortcuts(locale) {
                                        cell(ui, &keys, theme::semibold(13.0), palette.text);
                                        cell(
                                            ui,
                                            &description,
                                            theme::regular(13.5),
                                            palette.secondary,
                                        );
                                        ui.end_row();
                                    }
                                });
                        },
                    );
                    ui.add_space(16.0);
                    buttons(ui, |ui| {
                        if theme::pill_button(ui, &palette, &gettext(locale, "Done"), true).clicked() {
                            app.actions.push(Action::CloseDialog);
                        }
                    });
                }
                Dialog::PremiumNeeded => {
                    theme::text(
                        ui,
                        gettext(locale, "This account cannot play music here"),
                        theme::bold(20.0),
                        palette.text,
                    );
                    ui.add_space(8.0);
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(gettext(
                                locale,
                                "Playback needs Spotify Premium. Free accounts can browse and search, but cannot play music through Spotifast.",
                            ))
                            .font(theme::regular(14.0))
                            .color(palette.secondary),
                        )
                        .wrap(),
                    );
                    ui.add_space(20.0);
                    buttons(ui, |ui| {
                        if theme::pill_button(ui, &palette, &gettext(locale, "OK"), true).clicked() {
                            app.actions.push(Action::CloseDialog);
                        }
                    });
                }
            }
        });
    app.dialog_rect = Some(response.response.rect);
    if response.should_close() {
        app.actions.push(Action::CloseDialog);
    }
}

/// The space kept clear around the large artwork, and below it for the
/// Close link.
const ARTWORK_MARGIN: f32 = 48.0;
const ARTWORK_CLOSE_ROOM: f32 = 40.0;
/// The longest side the large artwork is drawn at: Spotify serves covers
/// and photos 640 pixels across, so anything larger only blurs them.
const ARTWORK_LONGEST: f32 = 640.0;

/// An album's cover or an artist's photo, as large as the window allows up
/// to the size Spotify serves it at, over the dimmed page. A click on it or
/// around it, Close, or Escape puts it away.
fn artwork(app: &mut App, ctx: &egui::Context, picture: &Artwork) {
    let palette = app.palette;
    let locale = app.locale;
    let room = ctx.content_rect().size()
        - egui::vec2(
            2.0 * ARTWORK_MARGIN,
            2.0 * ARTWORK_MARGIN + ARTWORK_CLOSE_ROOM,
        );
    let size = artwork_size(room, picture.aspect);
    let mut close = false;
    let response = egui::Modal::new(egui::Id::new("artwork"))
        .frame(Frame::NONE)
        .backdrop_color(egui::Color32::from_black_alpha(200))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 16.0;
            ui.vertical_centered(|ui| {
                let (rect, shown) = ui.allocate_exact_size(size, egui::Sense::click());
                super::widgets::paint_shadow(ui, &palette, rect, 8.0);
                super::widgets::paint_cover_with_thumbnail(
                    ui,
                    &palette,
                    super::widgets::CoverSources {
                        requested: Some(&picture.url),
                        previous: picture.shown.as_deref(),
                        ..Default::default()
                    },
                    rect,
                    8.0,
                    theme::Icon::Music,
                    Some(app.backend.art()),
                );
                shown.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Image, true, picture.label.as_str())
                });
                close |= shown.clicked();
                close |= theme::link(
                    ui,
                    gettext(locale, "Close"),
                    theme::semibold(14.0),
                    egui::Color32::WHITE,
                )
                .clicked();
            });
        });
    app.dialog_rect = Some(response.response.rect);
    if close || response.should_close() {
        app.actions.push(Action::CloseDialog);
    }
}

/// The size the large artwork is drawn at: its own shape, within `room`,
/// and no longer than [`ARTWORK_LONGEST`] on its longer side.
fn artwork_size(room: egui::Vec2, aspect: f32) -> egui::Vec2 {
    let aspect = if aspect.is_finite() && aspect > 0.0 {
        aspect
    } else {
        1.0
    };
    let longest = if aspect >= 1.0 {
        ARTWORK_LONGEST
    } else {
        ARTWORK_LONGEST * aspect
    };
    let width = room.x.min(room.y * aspect).min(longest).max(64.0);
    egui::vec2(width, width / aspect)
}

// Keep button rows content-sized when the shared modal area last held a taller dialog.
fn buttons(ui: &mut egui::Ui, add_buttons: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
        Layout::right_to_left(Align::Center),
        add_buttons,
    );
}

fn duplicate_message(
    locale: Locale,
    playlist_name: &str,
    items: &[crate::api::models::PlayableItem],
    duplicate_uris: &[String],
) -> String {
    let mut seen = std::collections::HashSet::new();
    let names: Vec<&str> = items
        .iter()
        .filter(|item| duplicate_uris.iter().any(|uri| uri == item.uri()))
        .map(crate::api::models::PlayableItem::name)
        .filter(|name| seen.insert(*name))
        .collect();
    let one_item = items.len() == 1;
    let message = match names.as_slice() {
        [] if one_item => {
            // Translators: {playlist} is a playlist name.
            gettext(
                locale,
                "This song is already in “{playlist}”. Add it again?",
            )
        }
        [] => {
            // Translators: {playlist} is a playlist name.
            gettext(
                locale,
                "This song is already in “{playlist}”. Add them anyway?",
            )
        }
        [name] if one_item => {
            // Translators: {name} is a song name and {playlist} a playlist name.
            gettext(locale, "“{name}” is already in “{playlist}”. Add it again?")
                .replace("{name}", name)
                .into()
        }
        [name] => {
            // Translators: {name} is a song name and {playlist} a playlist name.
            gettext(
                locale,
                "“{name}” is already in “{playlist}”. Add them anyway?",
            )
            .replace("{name}", name)
            .into()
        }
        [first, second] => {
            // Translators: {first} and {second} are song names, {playlist} a playlist name.
            gettext(
                locale,
                "“{first}” and “{second}” are already in “{playlist}”. Add them anyway?",
            )
            .replace("{first}", first)
            .replace("{second}", second)
            .into()
        }
        [first, second, rest @ ..] => ngettext(
            locale,
            // Translators: {first} and {second} are song names, {count} is how many more songs are already in the playlist, and {playlist} is a playlist name.
            "“{first}”, “{second}”, and {count} more are already in “{playlist}”. Add them anyway?",
            "“{first}”, “{second}”, and {count} more are already in “{playlist}”. Add them anyway?",
            rest.len() as u32,
        )
        .replace("{first}", first)
        .replace("{second}", second)
        .replace("{count}", &rest.len().to_string())
        .into(),
    };
    message.replace("{playlist}", playlist_name)
}

fn text_field(
    ui: &mut egui::Ui,
    palette: &theme::Palette,
    locale: crate::i18n::Locale,
    id: &str,
    text: &mut String,
    hint: &str,
    focus: bool,
) -> egui::Response {
    let response = Frame::new()
        .fill(palette.surface)
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            super::widgets::text_edit(
                ui,
                locale,
                egui::TextEdit::singleline(text)
                    .id(egui::Id::new(id))
                    .hint_text(egui::RichText::new(hint).color(palette.dim))
                    .font(theme::regular(14.0))
                    .frame(egui::Frame::NONE)
                    .desired_width(f32::INFINITY),
            )
        })
        .inner;
    if focus && ui.memory(|memory| memory.focused().is_none()) {
        response.request_focus();
    }
    response
}

fn create_playlist(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let busy = app.playlist_busy;
    let Some(Dialog::CreatePlaylist {
        name,
        public,
        add_uris,
    }) = &mut app.dialog
    else {
        return;
    };
    theme::text(
        ui,
        gettext(locale, "New playlist"),
        theme::bold(20.0),
        palette.text,
    );
    ui.add_space(12.0);
    theme::text(
        ui,
        gettext(locale, "Name"),
        theme::medium(13.0),
        palette.secondary,
    );
    let field = text_field(
        ui,
        &palette,
        locale,
        "playlist-name",
        name,
        &gettext(locale, "My playlist"),
        true,
    );
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        super::widgets::switch(ui, &palette, &gettext(locale, "Public playlist"), public);
        theme::text(
            ui,
            gettext(locale, "Public playlist"),
            theme::regular(14.0),
            palette.text,
        );
    });
    if !add_uris.is_empty() {
        ui.add_space(6.0);
        let count = add_uris.len();
        theme::text(
            ui,
            ngettext(
                locale,
                // Translators: {count} is the number of songs.
                "{count} song will be added.",
                "{count} songs will be added.",
                count as u32,
            )
            .replace("{count}", &count.to_string()),
            theme::regular(13.0),
            palette.secondary,
        );
    }
    ui.add_space(20.0);
    let submit = field.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    let name_value = name.trim().to_string();
    let public_value = *public;
    let uris = add_uris.clone();
    buttons(ui, |ui| {
        if busy {
            theme::spinner(ui, 18.0, palette.accent);
        } else {
            let create = theme::pill_button(ui, &palette, &gettext(locale, "Create"), true)
                .clicked()
                || submit;
            if create && !name_value.is_empty() {
                app.actions.push(Action::CreatePlaylist {
                    name: name_value.clone(),
                    public: public_value,
                    add_uris: uris.clone(),
                });
            }
            if theme::pill_button(ui, &palette, &gettext(locale, "Cancel"), false).clicked() {
                app.actions.push(Action::CloseDialog);
            }
        }
    });
}

fn edit_playlist(app: &mut App, ui: &mut egui::Ui) {
    let existing_image = if let Some(Dialog::EditPlaylist { id, .. }) = &app.dialog {
        app.library
            .playlists
            .get()
            .and_then(|playlists| playlists.iter().find(|playlist| &playlist.id == id))
            .and_then(|playlist| crate::api::models::pick_image(&playlist.images, 100))
            .map(str::to_owned)
    } else {
        None
    };
    let palette = app.palette;
    let locale = app.locale;
    let busy = app.playlist_busy;
    let Some(Dialog::EditPlaylist {
        cover,
        id,
        name,
        description,
        public,
    }) = &mut app.dialog
    else {
        return;
    };
    theme::text(
        ui,
        gettext(locale, "Edit details"),
        theme::bold(20.0),
        palette.text,
    );
    ui.add_space(12.0);
    egui::ScrollArea::vertical()
        .max_height((ui.ctx().content_rect().height() - 210.0).max(100.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(selected) = &cover.selection {
                    ui.ctx()
                        .include_bytes(selected.uri.clone(), selected.jpeg.clone());
                    ui.add(
                        egui::Image::new(selected.uri.clone())
                            .texture_options(crate::images::COVER_TEXTURE)
                            .fit_to_exact_size(egui::vec2(72.0, 72.0))
                            .alt_text(gettext(locale, "Selected playlist cover")),
                    );
                } else if let Some(url) = &existing_image {
                    ui.add(
                        egui::Image::new(url)
                            .texture_options(crate::images::COVER_TEXTURE)
                            .fit_to_exact_size(egui::vec2(72.0, 72.0))
                            .alt_text(gettext(locale, "Current playlist cover")),
                    );
                }
                ui.vertical(|ui| {
                    if cover.request.is_some() || cover.uploading.is_some() {
                        theme::spinner(ui, 18.0, palette.accent);
                        theme::text(
                            ui,
                            if cover.uploading.is_some() {
                                gettext(locale, "Uploading cover…")
                            } else {
                                gettext(locale, "Choosing cover…")
                            },
                            theme::regular(13.0),
                            palette.secondary,
                        );
                    } else {
                        if theme::pill_button(ui, &palette, &gettext(locale, "Change cover"), false)
                            .clicked()
                        {
                            app.actions.push(Action::ChoosePlaylistCover(id.clone()));
                        }
                        if cover.selection.is_some()
                            && theme::pill_button(
                                ui,
                                &palette,
                                &gettext(locale, "Upload cover"),
                                true,
                            )
                            .clicked()
                        {
                            app.actions.push(Action::UploadPlaylistCover(id.clone()));
                        }
                    }
                });
            });
            if let Some(error) = &cover.error {
                ui.add(egui::Label::new(egui::RichText::new(error).color(palette.text)).wrap());
            }
            ui.add_space(10.0);
            theme::text(
                ui,
                gettext(locale, "Name"),
                theme::medium(13.0),
                palette.secondary,
            );
            text_field(
                ui,
                &palette,
                locale,
                "edit-name",
                name,
                &gettext(locale, "Playlist name"),
                true,
            );
            ui.add_space(10.0);
            theme::text(
                ui,
                gettext(locale, "Description"),
                theme::medium(13.0),
                palette.secondary,
            );
            Frame::new()
                .fill(palette.surface)
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    super::widgets::text_edit(
                        ui,
                        locale,
                        egui::TextEdit::multiline(description)
                            .id(egui::Id::new("edit-description"))
                            .hint_text(
                                egui::RichText::new(gettext(locale, "Optional description"))
                                    .color(palette.dim),
                            )
                            .font(theme::regular(14.0))
                            .frame(egui::Frame::NONE)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );
                });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                // Unknown shows as off; only a change of the switch is sent, so
                // a playlist nothing has described keeps whatever it was.
                let mut shown = public.unwrap_or(false);
                if super::widgets::switch(
                    ui,
                    &palette,
                    &gettext(locale, "Public playlist"),
                    &mut shown,
                )
                .changed()
                {
                    *public = Some(shown);
                }
                theme::text(
                    ui,
                    gettext(locale, "Public playlist"),
                    theme::regular(14.0),
                    palette.text,
                );
            });
        });
    ui.add_space(20.0);
    let id = id.clone();
    let busy = busy || cover.uploading.is_some() || cover.request.is_some();
    let name_value = name.trim().to_string();
    let description_value = description.trim().to_string();
    let public_value = *public;
    buttons(ui, |ui| {
        if busy {
            theme::spinner(ui, 18.0, palette.accent);
        } else {
            if theme::pill_button(ui, &palette, &gettext(locale, "Save"), true).clicked()
                && !name_value.is_empty()
            {
                app.actions.push(Action::UpdatePlaylist {
                    id: id.clone(),
                    name: name_value.clone(),
                    description: description_value.clone(),
                    public: public_value,
                });
            }
            if theme::pill_button(ui, &palette, &gettext(locale, "Cancel"), false).clicked() {
                app.actions.push(Action::CloseDialog);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{artwork_size, duplicate_message};
    use crate::api::models::{PlayableItem, Track};
    use crate::i18n::Locale;

    fn song(uri: &str, name: &str) -> PlayableItem {
        PlayableItem::Track(Track {
            uri: uri.into(),
            name: name.into(),
            ..Default::default()
        })
    }

    #[test]
    fn duplicate_dialog_names_the_song() {
        let items = vec![song("spotify:track:honey", "Honey")];
        let message = duplicate_message(
            Locale::English,
            "The best music ever",
            &items,
            &["spotify:track:honey".into()],
        );

        assert_eq!(
            message,
            "“Honey” is already in “The best music ever”. Add it again?"
        );
    }

    #[test]
    fn duplicate_dialog_names_only_the_duplicates_in_a_selection() {
        let items = vec![
            song("spotify:track:honey", "Honey"),
            song("spotify:track:new", "New song"),
        ];
        let message = duplicate_message(
            Locale::English,
            "The best music ever",
            &items,
            &["spotify:track:honey".into()],
        );

        assert_eq!(
            message,
            "“Honey” is already in “The best music ever”. Add them anyway?"
        );
    }

    /// The large artwork keeps its shape, fills a small window, and stops
    /// at the size Spotify serves it at in a large one.
    #[test]
    fn large_artwork_fits_the_window_and_never_blows_up() {
        let square = artwork_size(egui::vec2(2000.0, 1500.0), 1.0);
        assert_eq!(square, egui::vec2(640.0, 640.0));
        let short = artwork_size(egui::vec2(900.0, 400.0), 1.0);
        assert_eq!(short, egui::vec2(400.0, 400.0));
        let wide = artwork_size(egui::vec2(2000.0, 1500.0), 2.0);
        assert_eq!(wide, egui::vec2(640.0, 320.0));
        let tall = artwork_size(egui::vec2(2000.0, 1500.0), 0.5);
        assert_eq!(tall, egui::vec2(320.0, 640.0));
        let nonsense = artwork_size(egui::vec2(300.0, 300.0), f32::NAN);
        assert_eq!(nonsense, egui::vec2(300.0, 300.0));
    }
}
