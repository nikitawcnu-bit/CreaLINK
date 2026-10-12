
use macroquad::prelude::*;
use rhai::Engine;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FONT_PATH: &str = "C:/Windows/Fonts/CascadiaMono.ttf";
const SAVE_FILE: &str = "crealink_project.txt";
const ACCOUNT_FILE: &str = "crealink_account.txt";
const SKIN_DIR: &str = "skins";

const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_RADIUS: f32 = 0.35;
const WALK_SPEED: f32 = 5.0;
const RUN_SPEED: f32 = 9.0;
const JUMP_SPEED: f32 = 7.0;
const GRAVITY: f32 = 18.0;

#[derive(Clone)]
struct Part {
    name: String,
    position: Vec3,
    size: Vec3,
    rotation: Vec3,
    color: Color,
}

struct Player {
    position: Vec3,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    grounded: bool,
}

#[derive(Clone)]
struct Account {
    username: String,
    id: String,
    skin_name: String,
    skin_source: String,
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Menu,
    GameBrowser,
    Settings,
    Language,
    Display,
    Background,
    Account,
    Editor,
}

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Beige,
    Light,
    Dark,
    Blue,
    Green,
    Purple,
}

struct App {
    page: Page,
    previous_page: Page,

    theme: Theme,
    language: usize,
    fullscreen: bool,

    parts: Vec<Part>,
    selected: Option<usize>,

    camera: Camera3D,
    camera_target: Vec3,
    camera_distance: f32,
    camera_yaw: f32,
    camera_pitch: f32,
    orbiting: bool,

    playing: bool,
    player: Player,
    mouse_captured: bool,

    font: Option<Font>,

    status: String,
    button_anim: HashMap<String, f32>,
    page_transition: f32,
    menu_time: f32,

    account: Account,
    username_edit: String,
    skin_url_edit: String,
    input_focus: InputFocus,

    skin_texture: Option<Texture2D>,
    skin_path: Option<PathBuf>,
    skin_loading: bool,

    last_mouse: Vec2,
    mouse_initialized: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum InputFocus {
    None,
    Username,
    SkinUrl,
}

#[derive(Clone)]
struct GameEntry {
    name: String,
    author: String,
    description: String,
    local: bool,
}

#[macroquad::main("CreaLink")]
async fn main() {
    let font = load_ttf_font(FONT_PATH).await.ok();

    let account = load_account();

    let mut app = App {
        page: Page::Menu,
        previous_page: Page::Menu,
        theme: Theme::Beige,
        language: 0,
        fullscreen: false,

        parts: Vec::new(),
        selected: None,

        camera: Camera3D {
            position: vec3(7.0, 6.0, 9.0),
            target: vec3(0.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 45.0,
            aspect: None,
            projection: Projection::Perspective,
            ..Default::default()
        },
        camera_target: vec3(0.0, 0.0, 0.0),
        camera_distance: 12.0,
        camera_yaw: 0.7,
        camera_pitch: 0.45,
        orbiting: false,

        playing: false,
        player: Player {
            position: vec3(0.0, PLAYER_HEIGHT, 5.0),
            velocity: Vec3::ZERO,
            yaw: std::f32::consts::PI,
            pitch: 0.0,
            grounded: false,
        },
        mouse_captured: false,

        font,
        status: "CreaLink ready".to_string(),
        button_anim: HashMap::new(),
        page_transition: 0.0,
        menu_time: 0.0,

        username_edit: account.username.clone(),
        skin_url_edit: String::new(),
        input_focus: InputFocus::None,

        account,
        skin_texture: None,
        skin_path: None,
        skin_loading: false,

        last_mouse: vec2(mouse_position().0, mouse_position().1),
        mouse_initialized: false,
    };

    add_part(
        &mut app,
        "Ground",
        vec3(0.0, -0.5, 0.0),
        vec3(12.0, 1.0, 12.0),
        Color::new(0.62, 0.57, 0.48, 1.0),
    );

    load_current_skin(&mut app).await;
    run_rhai_test();

    loop {
        let dt = get_frame_time().min(0.05);
        clear_background(theme_background(app.theme));

        app.menu_time += dt;
        app.page_transition = (app.page_transition - dt * 3.5).max(0.0);

        match app.page {
            Page::Menu => draw_menu(&mut app, dt),
            Page::GameBrowser => draw_game_browser(&mut app, dt),
            Page::Settings => draw_settings(&mut app, dt),
            Page::Language => draw_language(&mut app, dt),
            Page::Display => draw_display(&mut app, dt),
            Page::Background => draw_background_page(&mut app, dt),
            Page::Account => draw_account(&mut app, dt).await,
            Page::Editor => {
                editor_input(&mut app, dt);

                if app.playing {
                    draw_play_mode(&mut app);
                } else {
                    draw_editor_world(&mut app);
                    draw_editor_ui(&mut app, dt);
                }
            }
        }

        draw_transition(&app);
        next_frame().await;
    }
}

fn tr(language: usize, key: &str) -> &str {
    match language {
        0 => match key {
            "create" => "Создать",
            "play" => "Играть",
            "settings" => "Настройки",
            "exit" => "Выход",
            "create_build_play" => "Создавай. Собирай. Играй.",
            "language" => "Язык",
            "display" => "Экран",
            "background" => "Фон",
            "account" => "Аккаунт и скин",
            "configure_environment" => "Настрой окружение CreaLink",
            "choose_language" => "Выбери язык интерфейса",
            "window_settings" => "Настройки окна",
            "theme_select" => "Выбери фон",
            "back" => "Назад",
            "fullscreen_on" => "Полноэкранный режим: ВКЛ",
            "fullscreen_off" => "Полноэкранный режим: ВЫКЛ",
            "beige" => "Бежевый",
            "light" => "Светлый",
            "dark" => "Тёмный",
            "blue" => "Синий",
            "green" => "Зелёный",
            "purple" => "Фиолетовый",
            "file" => "Файл",
            "part_add" => "Добавить",
            "duplicate" => "Дублировать",
            "delete" => "Удалить",
            "stop" => "Стоп",
            "build" => "Сборка",
            "explorer" => "Объекты",
            "inspector" => "Инспектор",
            "nothing_selected" => "Ничего не выбрано",
            "position" => "Позиция",
            "size" => "Размер",
            "rotation" => "Поворот",
            "move" => "WASD: движение",
            "vertical" => "Q/E: вверх/вниз",
            "rotate" => "R: вращение",
            "play_mode" => "Режим игры",
            "save" => "Сохранить",
            "load" => "Загрузить",
            "menu" => "Меню",
            "run" => "Запуск",
            "jump" => "Пробел: прыжок",
            "look" => "Мышь: обзор",
            "exit_play" => "Esc: выйти из игры",
            "click_to_look" => "Кликни по окну для управления мышью",
            "play_controls" => "WASD: движение • Shift: бег • Space: прыжок",
            "part_created" => "Объект создан",
            "part_deleted" => "Объект удалён",
            "part_duplicated" => "Объект дублирован",
            "ground_protected" => "Ground нельзя удалить",
            "project_saved" => "Проект сохранён",
            "project_loaded" => "Проект загружен",
            "project_empty" => "Сохранение не найдено",
            "language_russian" => "Русский",
            "language_english" => "English",
            "language_german" => "Deutsch",
            "language_changed" => "Язык изменён",
            "save_error" => "Ошибка сохранения",
            "load_error" => "Ошибка загрузки",
            "play_started" => "Игра запущена",
            "play_stopped" => "Игра остановлена",
            "build_mode" => "Режим строительства",
            "games" => "Игры",
            "browser_hint" => "Здесь будут опубликованные игры CreaLink",
            "local_demo" => "Локальный проект",
            "p2p_demo" => "P2P игра",
            "download_p2p" => "P2P-загрузка пока не подключена",
            "account_title" => "Аккаунт и скин",
            "username" => "Имя игрока",
            "save_account" => "Сохранить профиль",
            "skin_builtin" => "Встроенные скины",
            "skin_internet" => "Скин из интернета",
            "skin_url" => "URL PNG-скина",
            "download_skin" => "Загрузить скин",
            "skin_default" => "Классический",
            "skin_robot" => "Робот",
            "skin_void" => "Пустотный",
            "skin_ready" => "Скин применён",
            "skin_error" => "Не удалось загрузить скин",
            "local_account" => "Локальный аккаунт",
            "account_saved" => "Профиль сохранён",
            "type_here" => "Нажми поле и печатай",
            _ => key,
        },
        2 => match key {
            "create" => "Erstellen",
            "play" => "Spielen",
            "settings" => "Einstellungen",
            "exit" => "Beenden",
            "create_build_play" => "Erstellen. Bauen. Spielen.",
            "language" => "Sprache",
            "display" => "Anzeige",
            "background" => "Hintergrund",
            "account" => "Konto & Skin",
            "back" => "Zurück",
            "language_russian" => "Russisch",
            "language_english" => "Englisch",
            "language_german" => "Deutsch",
            "games" => "Spiele",
            "browser_hint" => "Hier erscheinen veröffentlichte CreaLink-Spiele",
            "account_title" => "Konto & Skin",
            "username" => "Spielername",
            "save_account" => "Profil speichern",
            "skin_builtin" => "Eingebaute Skins",
            "skin_internet" => "Skin aus dem Internet",
            "skin_url" => "Skin-PNG-URL",
            "download_skin" => "Skin laden",
            "local_account" => "Lokales Konto",
            _ => key,
        },
        _ => match key {
            "create" => "Create",
            "play" => "Play",
            "settings" => "Settings",
            "exit" => "Exit",
            "create_build_play" => "Create. Build. Play.",
            "language" => "Language",
            "display" => "Display",
            "background" => "Background",
            "account" => "Account & Skin",
            "configure_environment" => "Configure CreaLink",
            "choose_language" => "Choose interface language",
            "window_settings" => "Window settings",
            "theme_select" => "Choose background",
            "back" => "Back",
            "fullscreen_on" => "Fullscreen: ON",
            "fullscreen_off" => "Fullscreen: OFF",
            "beige" => "Beige",
            "light" => "Light",
            "dark" => "Dark",
            "blue" => "Blue",
            "green" => "Green",
            "purple" => "Purple",
            "file" => "File",
            "part_add" => "Add",
            "duplicate" => "Duplicate",
            "delete" => "Delete",
            "stop" => "Stop",
            "build" => "Build",
            "explorer" => "Explorer",
            "inspector" => "Inspector",
            "nothing_selected" => "Nothing selected",
            "position" => "Position",
            "size" => "Size",
            "rotation" => "Rotation",
            "move" => "WASD: move",
            "vertical" => "Q/E: up/down",
            "rotate" => "R: rotate",
            "play_mode" => "Play mode",
            "save" => "Save",
            "load" => "Load",
            "menu" => "Menu",
            "run" => "Run",
            "jump" => "Space: jump",
            "look" => "Mouse: look",
            "exit_play" => "Esc: exit play",
            "click_to_look" => "Click the window to capture the mouse",
            "play_controls" => "WASD: move • Shift: run • Space: jump",
            "part_created" => "Part created",
            "part_deleted" => "Part deleted",
            "part_duplicated" => "Part duplicated",
            "ground_protected" => "Ground cannot be deleted",
            "project_saved" => "Project saved",
            "project_loaded" => "Project loaded",
            "project_empty" => "Save not found",
            "language_russian" => "Russian",
            "language_english" => "English",
            "language_german" => "German",
            "language_changed" => "Language changed",
            "save_error" => "Save error",
            "load_error" => "Load error",
            "play_started" => "Game started",
            "play_stopped" => "Game stopped",
            "build_mode" => "Build mode",
            "games" => "Games",
            "browser_hint" => "Published CreaLink games will appear here",
            "local_demo" => "Local project",
            "p2p_demo" => "P2P game",
            "download_p2p" => "P2P download is not connected yet",
            "account_title" => "Account & Skin",
            "username" => "Player name",
            "save_account" => "Save profile",
            "skin_builtin" => "Built-in skins",
            "skin_internet" => "Skin from Internet",
            "skin_url" => "Skin PNG URL",
            "download_skin" => "Download skin",
            "skin_default" => "Classic",
            "skin_robot" => "Robot",
            "skin_void" => "Void",
            "skin_ready" => "Skin applied",
            "skin_error" => "Could not load skin",
            "local_account" => "Local account",
            "account_saved" => "Profile saved",
            "type_here" => "Click a field and type",
            _ => key,
        },
    }
}

fn theme_background(theme: Theme) -> Color {
    match theme {
        Theme::Beige => Color::new(0.89, 0.84, 0.74, 1.0),
        Theme::Light => Color::new(0.96, 0.96, 0.94, 1.0),
        Theme::Dark => Color::new(0.075, 0.08, 0.095, 1.0),
        Theme::Blue => Color::new(0.10, 0.16, 0.24, 1.0),
        Theme::Green => Color::new(0.10, 0.18, 0.13, 1.0),
        Theme::Purple => Color::new(0.16, 0.11, 0.20, 1.0),
    }
}

fn theme_text(theme: Theme) -> Color {
    match theme {
        Theme::Beige => Color::new(0.15, 0.13, 0.11, 1.0),
        Theme::Light => Color::new(0.10, 0.10, 0.11, 1.0),
        _ => Color::new(0.92, 0.93, 0.96, 1.0),
    }
}

fn theme_muted(theme: Theme) -> Color {
    let c = theme_text(theme);
    Color::new(c.r, c.g, c.b, 0.62)
}

fn theme_accent(theme: Theme) -> Color {
    match theme {
        Theme::Beige => Color::new(0.34, 0.25, 0.16, 1.0),
        Theme::Light => Color::new(0.20, 0.32, 0.55, 1.0),
        Theme::Dark => Color::new(0.55, 0.72, 1.0, 1.0),
        Theme::Blue => Color::new(0.45, 0.72, 1.0, 1.0),
        Theme::Green => Color::new(0.47, 0.88, 0.62, 1.0),
        Theme::Purple => Color::new(0.80, 0.60, 1.0, 1.0),
    }
}

fn draw_text_custom(app: &App, text: &str, x: f32, y: f32, size: f32, color: Color) {
    if let Some(font) = &app.font {
        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font: Some(font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    } else {
        draw_text(text, x, y, size, color);
    }
}

fn measure_text_custom(app: &App, text: &str, size: f32) -> f32 {
    if let Some(font) = &app.font {
        measure_text(text, Some(font), size as u16, 1.0).width
    } else {
        measure_text(text, None, size as u16, 1.0).width
    }
}

fn centered_text(app: &App, text: &str, y: f32, size: f32, color: Color) {
    let w = measure_text_custom(app, text, size);
    draw_text_custom(app, text, (screen_width() - w) * 0.5, y, size, color);
}

fn ease(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn animated_button(
    app: &mut App,
    id: &str,
    label: &str,
    x: f32,
    y: f32,
    size: f32,
    dt: f32,
) -> bool {
    let (mx, my) = mouse_position();
    let w = measure_text_custom(app, label, size);

    let hovered = mx >= x - 18.0
        && mx <= x + w + 18.0
        && my >= y - size - 6.0
        && my <= y + 12.0;

    let value = app.button_anim.entry(id.to_string()).or_insert(0.0);
    let target = if hovered { 1.0 } else { 0.0 };

    *value += (target - *value) * (dt * 14.0).min(1.0);
    let a = ease(*value);

    let text_color = theme_text(app.theme);
    let accent = theme_accent(app.theme);

    let color = Color::new(
        text_color.r + (accent.r - text_color.r) * a,
        text_color.g + (accent.g - text_color.g) * a,
        text_color.b + (accent.b - text_color.b) * a,
        0.55 + a * 0.45,
    );

    let slide = a * 8.0;

    if a > 0.01 {
        draw_text_custom(
            app,
            ">",
            x - 15.0 + slide * 0.35,
            y,
            size,
            Color::new(accent.r, accent.g, accent.b, a),
        );
    }

    draw_text_custom(app, label, x + slide, y, size, color);

    if a > 0.01 {
        let underline_width = w * a + 5.0;
        draw_line(
            x + slide,
            y + 7.0,
            x + slide + underline_width,
            y + 7.0,
            2.0,
            Color::new(accent.r, accent.g, accent.b, a),
        );
    }

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn small_button(app: &mut App, id: &str, label: &str, x: f32, y: f32, dt: f32) -> bool {
    animated_button(app, id, label, x, y, 17.0, dt)
}

fn navigate(app: &mut App, page: Page) {
    app.previous_page = app.page;
    app.page = page;
    app.page_transition = 1.0;
    app.input_focus = InputFocus::None;
}

fn draw_transition(app: &App) {
    if app.page_transition <= 0.0 {
        return;
    }
    let a = ease(app.page_transition) * 0.12;
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, a),
    );
}

fn draw_menu(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);
    let accent = theme_accent(app.theme);

    let intro = ease((app.menu_time * 1.8).min(1.0));
    let offset = (1.0 - intro) * -24.0;

    draw_text_custom(
        app,
        "CreaLink",
        58.0 + offset,
        92.0,
        62.0,
        Color::new(text.r, text.g, text.b, intro),
    );

    draw_text_custom(
        app,
        tr(app.language, "create_build_play"),
        61.0 + offset,
        123.0,
        16.0,
        Color::new(muted.r, muted.g, muted.b, intro),
    );

    let x = 78.0;
    let mut y = 205.0;

    if animated_button(app, "menu_create", tr(app.language, "create"), x, y, 25.0, dt) {
        navigate(app, Page::Editor);
    }
    y += 54.0;

    if animated_button(app, "menu_play", tr(app.language, "play"), x, y, 25.0, dt) {
        navigate(app, Page::GameBrowser);
    }
    y += 54.0;

    if animated_button(app, "menu_settings", tr(app.language, "settings"), x, y, 25.0, dt) {
        navigate(app, Page::Settings);
    }
    y += 54.0;

    if animated_button(app, "menu_exit", tr(app.language, "exit"), x, y, 25.0, dt) {
        std::process::exit(0);
    }

    draw_text_custom(
        app,
        &format!("{}  •  {}", tr(app.language, "local_account"), app.account.username),
        28.0,
        screen_height() - 28.0,
        14.0,
        muted,
    );

    draw_line(
        28.0,
        screen_height() - 52.0,
        220.0,
        screen_height() - 52.0,
        2.0,
        Color::new(accent.r, accent.g, accent.b, 0.25),
    );
}

fn draw_game_browser(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);
    let accent = theme_accent(app.theme);

    draw_text_custom(app, tr(app.language, "games"), 55.0, 78.0, 42.0, text);
    draw_text_custom(
        app,
        tr(app.language, "browser_hint"),
        58.0,
        108.0,
        15.0,
        muted,
    );

    let games = vec![
        GameEntry {
            name: tr(app.language, "local_demo").to_string(),
            author: app.account.username.clone(),
            description: "Твой текущий локальный CreaLink проект".to_string(),
            local: true,
        },
        GameEntry {
            name: "Voxel World".to_string(),
            author: "CreaLink Community".to_string(),
            description: "Демо-карта, которая будет передаваться через P2P".to_string(),
            local: false,
        },
        GameEntry {
            name: "Space Station".to_string(),
            author: "CreaLink Community".to_string(),
            description: "Пример опубликованной игры".to_string(),
            local: false,
        },
    ];

    let mut y = 155.0;
    for (i, game) in games.iter().enumerate() {
        let w = screen_width() - 120.0;
        draw_rectangle(
            55.0,
            y,
            w,
            92.0,
            Color::new(accent.r, accent.g, accent.b, 0.08),
        );
        draw_line(55.0, y + 91.0, 55.0 + w, y + 91.0, 1.0, Color::new(accent.r, accent.g, accent.b, 0.22));

        draw_text_custom(app, &game.name, 78.0, y + 30.0, 23.0, text);
        draw_text_custom(
            app,
            &format!("{}  •  {}", game.author, game.description),
            78.0,
            y + 55.0,
            13.0,
            muted,
        );

        let label = if game.local {
            tr(app.language, "play")
        } else {
            tr(app.language, "p2p_demo")
        };

        if small_button(app, &format!("game_{}", i), label, screen_width() - 205.0, y + 43.0, dt) {
            if game.local {
                start_play(app);
                navigate(app, Page::Editor);
            } else {
                app.status = tr(app.language, "download_p2p").to_string();
            }
        }

        y += 112.0;
    }

    if small_button(app, "browser_back", tr(app.language, "back"), 60.0, screen_height() - 35.0, dt) {
        navigate(app, Page::Menu);
    }
}

fn draw_settings(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);

    draw_text_custom(app, tr(app.language, "settings"), 55.0, 85.0, 42.0, text);
    draw_text_custom(
        app,
        tr(app.language, "configure_environment"),
        58.0,
        115.0,
        15.0,
        muted,
    );

    let x = 75.0;
    let mut y = 190.0;

    if animated_button(app, "settings_language", tr(app.language, "language"), x, y, 23.0, dt) {
        navigate(app, Page::Language);
    }
    y += 55.0;

    if animated_button(app, "settings_display", tr(app.language, "display"), x, y, 23.0, dt) {
        navigate(app, Page::Display);
    }
    y += 55.0;

    if animated_button(app, "settings_background", tr(app.language, "background"), x, y, 23.0, dt) {
        navigate(app, Page::Background);
    }
    y += 55.0;

    if animated_button(app, "settings_account", tr(app.language, "account"), x, y, 23.0, dt) {
        navigate(app, Page::Account);
    }
    y += 70.0;

    if animated_button(app, "settings_back", tr(app.language, "back"), x, y, 23.0, dt) {
        navigate(app, Page::Menu);
    }

    draw_text_custom(app, &app.status, 60.0, screen_height() - 30.0, 14.0, muted);
}

fn draw_language(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);

    draw_text_custom(app, tr(app.language, "language"), 55.0, 85.0, 42.0, text);
    draw_text_custom(app, tr(app.language, "choose_language"), 58.0, 115.0, 15.0, muted);

    let x = 75.0;
    let mut y = 190.0;

    for (id, label, lang) in [
        ("ru", tr(0, "language_russian"), 0usize),
        ("en", tr(1, "language_english"), 1usize),
        ("de", tr(2, "language_german"), 2usize),
    ] {
        if animated_button(app, &format!("lang_{}", id), label, x, y, 23.0, dt) {
            app.language = lang;
            app.status = tr(app.language, "language_changed").to_string();
        }
        y += 55.0;
    }

    if animated_button(app, "language_back", tr(app.language, "back"), x, y + 15.0, 23.0, dt) {
        navigate(app, Page::Settings);
    }
}

fn draw_display(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);

    draw_text_custom(app, tr(app.language, "display"), 55.0, 85.0, 42.0, text);
    draw_text_custom(app, tr(app.language, "window_settings"), 58.0, 115.0, 15.0, muted);

    let label = if app.fullscreen {
        tr(app.language, "fullscreen_on")
    } else {
        tr(app.language, "fullscreen_off")
    };

    if animated_button(app, "display_fullscreen", label, 75.0, 190.0, 23.0, dt) {
        app.fullscreen = !app.fullscreen;
        set_fullscreen(app.fullscreen);
    }

    if animated_button(app, "display_back", tr(app.language, "back"), 75.0, 270.0, 23.0, dt) {
        navigate(app, Page::Settings);
    }
}

fn draw_background_page(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);

    draw_text_custom(app, tr(app.language, "background"), 55.0, 85.0, 42.0, text);
    draw_text_custom(app, tr(app.language, "theme_select"), 58.0, 115.0, 15.0, muted);

    let themes = [
        ("beige", tr(app.language, "beige"), Theme::Beige),
        ("light", tr(app.language, "light"), Theme::Light),
        ("dark", tr(app.language, "dark"), Theme::Dark),
        ("blue", tr(app.language, "blue"), Theme::Blue),
        ("green", tr(app.language, "green"), Theme::Green),
        ("purple", tr(app.language, "purple"), Theme::Purple),
    ];

    let mut y = 170.0;
    for (id, label, theme) in themes {
        if animated_button(app, &format!("theme_{}", id), label, 75.0, y, 22.0, dt) {
            app.theme = theme;
        }
        y += 45.0;
    }

    if animated_button(app, "background_back", tr(app.language, "back"), 75.0, y + 10.0, 22.0, dt) {
        navigate(app, Page::Settings);
    }
}

async fn draw_account(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);
    let accent = theme_accent(app.theme);

    draw_text_custom(app, tr(app.language, "account_title"), 55.0, 75.0, 38.0, text);
    draw_text_custom(
        app,
        &format!("{}: {}", tr(app.language, "local_account"), app.account.id),
        58.0,
        103.0,
        13.0,
        muted,
    );

    draw_text_custom(app, tr(app.language, "username"), 65.0, 155.0, 17.0, text);
    draw_input(
        app,
        "username",
        &app.username_edit.clone(),
        65.0,
        170.0,
        360.0,
        42.0,
        InputFocus::Username,
    );

    if animated_button(
        app,
        "save_account",
        tr(app.language, "save_account"),
        65.0,
        250.0,
        19.0,
        dt,
    ) {
        let name = app.username_edit.trim();
        if !name.is_empty() {
            app.account.username = name.chars().take(24).collect();
            save_account(&app.account);
            app.status = tr(app.language, "account_saved").to_string();
        }
    }

    draw_text_custom(app, tr(app.language, "skin_builtin"), 65.0, 325.0, 18.0, text);

    let skin_names = [
        ("Classic", "classic"),
        ("Robot", "robot"),
        ("Void", "void"),
    ];

    for (i, (label, kind)) in skin_names.iter().enumerate() {
        if animated_button(
            app,
            &format!("skin_builtin_{}", kind),
            match *kind {
                "classic" => tr(app.language, "skin_default"),
                "robot" => tr(app.language, "skin_robot"),
                _ => tr(app.language, "skin_void"),
            },
            65.0,
            360.0 + i as f32 * 42.0,
            18.0,
            dt,
        ) {
            apply_builtin_skin(app, kind).await;
        }

        let preview_x = 250.0;
        let preview_y = 342.0 + i as f32 * 42.0;
        draw_rectangle(preview_x, preview_y, 20.0, 20.0, builtin_skin_color(kind));
    }

    draw_text_custom(app, tr(app.language, "skin_internet"), 430.0, 325.0, 18.0, text);
    draw_input(
        app,
        "skin_url",
        &app.skin_url_edit.clone(),
        430.0,
        340.0,
        screen_width() - 500.0,
        42.0,
        InputFocus::SkinUrl,
    );

    if animated_button(
        app,
        "download_skin",
        tr(app.language, "download_skin"),
        430.0,
        420.0,
        18.0,
        dt,
    ) {
        download_skin(app).await;
    }

    if let Some(texture) = &app.skin_texture {
        let size = 180.0;
        draw_rectangle(
            screen_width() - 245.0,
            120.0,
            190.0,
            235.0,
            Color::new(accent.r, accent.g, accent.b, 0.08),
        );
        draw_text_custom(app, &app.account.skin_name, screen_width() - 225.0, 150.0, 15.0, text);

        let params = DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            ..Default::default()
        };
        draw_texture_ex(
            texture,
            screen_width() - 235.0,
            165.0,
            WHITE,
            params,
        );
    }

    draw_text_custom(app, tr(app.language, "type_here"), 65.0, screen_height() - 62.0, 13.0, muted);
    draw_text_custom(app, &app.status, 65.0, screen_height() - 35.0, 14.0, muted);

    if animated_button(app, "account_back", tr(app.language, "back"), 65.0, 470.0, 18.0, dt) {
        navigate(app, Page::Settings);
    }
}

fn draw_input(
    app: &mut App,
    id: &str,
    value: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    focus: InputFocus,
) {
    let active = app.input_focus == focus;
    let accent = theme_accent(app.theme);
    let text = theme_text(app.theme);
    let border = if active { accent } else { Color::new(accent.r, accent.g, accent.b, 0.25) };

    draw_rectangle(
        x,
        y,
        w,
        h,
        Color::new(accent.r, accent.g, accent.b, if active { 0.10 } else { 0.05 }),
    );
    draw_rectangle_lines(x, y, w, h, 1.0, border);

    let shown = if value.is_empty() { "..." } else { value };
    draw_text_custom(app, shown, x + 10.0, y + 27.0, 15.0, text);

    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if mx >= x && mx <= x + w && my >= y && my <= y + h {
            app.input_focus = focus;
        }
    }

    if active {
        while let Some(c) = get_char_pressed() {
            if c != '\n' && c != '\r' && !c.is_control() {
                match focus {
                    InputFocus::Username => {
                        if app.username_edit.chars().count() < 24 {
                            app.username_edit.push(c);
                        }
                    }
                    InputFocus::SkinUrl => {
                        if app.skin_url_edit.chars().count() < 500 {
                            app.skin_url_edit.push(c);
                        }
                    }
                    InputFocus::None => {}
                }
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            match focus {
                InputFocus::Username => {
                    app.username_edit.pop();
                }
                InputFocus::SkinUrl => {
                    app.skin_url_edit.pop();
                }
                InputFocus::None => {}
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            app.input_focus = InputFocus::None;
        }
    }

    let _ = id;
}

fn editor_input(app: &mut App, dt: f32) {
    if is_key_pressed(KeyCode::Escape) {
        if app.playing {
            stop_play(app);
        } else {
            navigate(app, Page::Menu);
        }
        return;
    }

    if app.playing {
        if !app.mouse_captured && is_mouse_button_pressed(MouseButton::Left) {
            app.mouse_captured = true;
            app.last_mouse = vec2(mouse_position().0, mouse_position().1);
            app.mouse_initialized = true;
            show_mouse(false);
        }

        if is_key_pressed(KeyCode::P) {
            stop_play(app);
            return;
        }

        update_player_mouse(app);
        update_player(app, dt);
        return;
    }

    if is_key_pressed(KeyCode::P) {
        start_play(app);
        return;
    }

    if is_key_pressed(KeyCode::Delete) {
        delete_part(app);
    }

    if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::D) {
        duplicate_part(app);
    }

    if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::S) {
        save_project(app);
    }

    if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::L) {
        load_project(app);
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        select_part(app);
    }

    let (_, wheel_y) = mouse_wheel();
    if wheel_y.abs() > 0.01 {
        app.camera_distance = (app.camera_distance - wheel_y * 0.7).clamp(2.0, 40.0);
    }

    if is_mouse_button_down(MouseButton::Middle) {
        let delta = mouse_delta_manual(app);
        app.camera_yaw -= delta.x * 0.008;
        app.camera_pitch = (app.camera_pitch + delta.y * 0.008).clamp(-1.35, 1.35);
        app.orbiting = true;
    } else {
        app.orbiting = false;
    }

    if let Some(index) = app.selected {
        if index < app.parts.len() {
            let speed = if is_key_down(KeyCode::LeftShift) { 5.0 } else { 2.0 };
            let amount = speed * dt;

            let forward = editor_flat_forward(app);
            let right = vec3(forward.z, 0.0, -forward.x);

            if is_key_down(KeyCode::W) {
                app.parts[index].position += forward * amount;
            }
            if is_key_down(KeyCode::S) {
                app.parts[index].position -= forward * amount;
            }
            if is_key_down(KeyCode::A) {
                app.parts[index].position -= right * amount;
            }
            if is_key_down(KeyCode::D) {
                app.parts[index].position += right * amount;
            }
            if is_key_down(KeyCode::Q) {
                app.parts[index].position.y += amount;
            }
            if is_key_down(KeyCode::E) {
                app.parts[index].position.y -= amount;
            }
            if is_key_down(KeyCode::R) {
                app.parts[index].rotation.y += 1.8 * dt;
            }
        }
    }

    update_camera(app);
}

fn editor_flat_forward(app: &App) -> Vec3 {
    let mut forward = app.camera_target - app.camera.position;
    forward.y = 0.0;
    if forward.length_squared() < 0.0001 {
        vec3(0.0, 0.0, -1.0)
    } else {
        forward.normalize()
    }
}

fn update_camera(app: &mut App) {
    let cp = app.camera_pitch.cos();
    let sp = app.camera_pitch.sin();
    let cy = app.camera_yaw.cos();
    let sy = app.camera_yaw.sin();

    let direction = vec3(cy * cp, sp, sy * cp);
    app.camera.position = app.camera_target - direction * app.camera_distance;
    app.camera.target = app.camera_target;
    app.camera.up = vec3(0.0, 1.0, 0.0);
}

fn screen_ray(app: &App) -> (Vec3, Vec3) {
    let mouse = mouse_position();
    let w = screen_width().max(1.0);
    let h = screen_height().max(1.0);

    let x = (2.0 * mouse.0 / w) - 1.0;
    let y = 1.0 - (2.0 * mouse.1 / h);

    let forward = (app.camera.target - app.camera.position).normalize();
    let right = forward.cross(app.camera.up).normalize();
    let up = right.cross(forward).normalize();

    let fov = app.camera.fovy.to_radians();
    let aspect = w / h;
    let tan_half = (fov * 0.5).tan();

    let direction = (forward + right * (x * tan_half * aspect) + up * (y * tan_half)).normalize();
    (app.camera.position, direction)
}

fn ray_box(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut tmin = -f32::INFINITY;
    let mut tmax = f32::INFINITY;

    for axis in 0..3 {
        let o = origin[axis];
        let d = direction[axis];
        let mn = min[axis];
        let mx = max[axis];

        if d.abs() < 0.000001 {
            if o < mn || o > mx {
                return None;
            }
        } else {
            let mut t1 = (mn - o) / d;
            let mut t2 = (mx - o) / d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return None;
            }
        }
    }

    if tmax < 0.0 {
        None
    } else {
        Some(if tmin >= 0.0 { tmin } else { tmax })
    }
}

fn select_part(app: &mut App) {
    let (origin, direction) = screen_ray(app);
    let mut best: Option<(usize, f32)> = None;

    for (i, part) in app.parts.iter().enumerate() {
        let half = part.size * 0.5;
        let min = part.position - half;
        let max = part.position + half;

        if let Some(distance) = ray_box(origin, direction, min, max) {
            if best.map(|(_, d)| distance < d).unwrap_or(true) {
                best = Some((i, distance));
            }
        }
    }

    app.selected = best.map(|(i, _)| i);
}

fn add_part(app: &mut App, name: &str, position: Vec3, size: Vec3, color: Color) {
    app.parts.push(Part {
        name: name.to_string(),
        position,
        size,
        rotation: Vec3::ZERO,
        color,
    });
    app.selected = Some(app.parts.len() - 1);
}

fn duplicate_part(app: &mut App) {
    let Some(index) = app.selected else {
        return;
    };
    if index >= app.parts.len() {
        return;
    }

    let mut copy = app.parts[index].clone();
    copy.name = format!("{}_Copy", copy.name);
    copy.position.x += 1.0;
    app.parts.push(copy);
    app.selected = Some(app.parts.len() - 1);
    app.status = tr(app.language, "part_duplicated").to_string();
}

fn delete_part(app: &mut App) {
    let Some(index) = app.selected else {
        return;
    };
    if index >= app.parts.len() {
        return;
    }

    if app.parts[index].name == "Ground" {
        app.status = tr(app.language, "ground_protected").to_string();
        return;
    }

    app.parts.remove(index);
    app.selected = None;
    app.status = tr(app.language, "part_deleted").to_string();
}

fn save_project(app: &mut App) {
    let mut out = String::new();

    for p in &app.parts {
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
            p.name.replace('|', "_"),
            p.position.x,
            p.position.y,
            p.position.z,
            p.size.x,
            p.size.y,
            p.size.z,
            p.rotation.x,
            p.rotation.y,
            p.rotation.z,
            p.color.r,
            p.color.g,
            p.color.b
        ));
    }

    match fs::write(SAVE_FILE, out) {
        Ok(_) => app.status = tr(app.language, "project_saved").to_string(),
        Err(_) => app.status = tr(app.language, "save_error").to_string(),
    }
}

fn load_project(app: &mut App) {
    let Ok(data) = fs::read_to_string(SAVE_FILE) else {
        app.status = tr(app.language, "project_empty").to_string();
        return;
    };

    let mut loaded = Vec::new();

    for line in data.lines() {
        let values: Vec<&str> = line.split('|').collect();
        if values.len() < 13 {
            continue;
        }

        let f = |i: usize, default: f32| values[i].parse::<f32>().unwrap_or(default);

        loaded.push(Part {
            name: values[0].to_string(),
            position: vec3(f(1, 0.0), f(2, 0.0), f(3, 0.0)),
            size: vec3(f(4, 1.0), f(5, 1.0), f(6, 1.0)),
            rotation: vec3(f(7, 0.0), f(8, 0.0), f(9, 0.0)),
            color: Color::new(f(10, 0.5), f(11, 0.5), f(12, 0.5), 1.0),
        });
    }

    if !loaded.is_empty() {
        app.parts = loaded;
        app.selected = None;
        app.status = tr(app.language, "project_loaded").to_string();
    } else {
        app.status = tr(app.language, "project_empty").to_string();
    }
}

fn draw_editor_world(app: &mut App) {
    update_camera(app);
    set_camera(&app.camera);

    for (i, part) in app.parts.iter().enumerate() {
        let mut color = part.color;
        if Some(i) == app.selected {
            color = Color::new(
                (color.r + 0.35).min(1.0),
                (color.g + 0.35).min(1.0),
                (color.b + 0.35).min(1.0),
                1.0,
            );
        }

        draw_cube(part.position, part.size, None, color);
        draw_cube_wires(part.position, part.size, Color::new(0.05, 0.05, 0.05, 0.28));
    }

    set_default_camera();
}

fn draw_editor_ui(app: &mut App, dt: f32) {
    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);
    let accent = theme_accent(app.theme);

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        54.0,
        Color::new(theme_background(app.theme).r, theme_background(app.theme).g, theme_background(app.theme).b, 0.94),
    );
    draw_text_custom(app, "CreaLink Editor", 24.0, 35.0, 21.0, text);

    let buttons = [
        ("editor_add", tr(app.language, "part_add")),
        ("editor_duplicate", tr(app.language, "duplicate")),
        ("editor_delete", tr(app.language, "delete")),
        ("editor_save", tr(app.language, "save")),
        ("editor_load", tr(app.language, "load")),
        ("editor_play", tr(app.language, "run")),
        ("editor_menu", tr(app.language, "menu")),
    ];

    let mut x = 250.0;
    for (id, label) in buttons {
        if animated_button(app, id, label, x, 34.0, 15.0, dt) {
            match id {
                "editor_add" => {
                    add_part(
                        app,
                        "Part",
                        app.camera_target + vec3(0.0, 0.5, 0.0),
                        vec3(1.0, 1.0, 1.0),
                        Color::new(0.35, 0.60, 0.85, 1.0),
                    );
                    app.status = tr(app.language, "part_created").to_string();
                }
                "editor_duplicate" => duplicate_part(app),
                "editor_delete" => delete_part(app),
                "editor_save" => save_project(app),
                "editor_load" => load_project(app),
                "editor_play" => start_play(app),
                "editor_menu" => navigate(app, Page::Menu),
                _ => {}
            }
        }
        x += measure_text_custom(app, label, 15.0) + 28.0;
    }

    draw_rectangle(
        0.0,
        54.0,
        235.0,
        screen_height() - 54.0,
        Color::new(
            theme_background(app.theme).r,
            theme_background(app.theme).g,
            theme_background(app.theme).b,
            0.90,
        ),
    );

    draw_text_custom(app, tr(app.language, "explorer"), 22.0, 86.0, 18.0, text);
    let mut y = 115.0;

    for (i, part) in app.parts.iter().enumerate() {
        let selected = app.selected == Some(i);
        let c = if selected { accent } else { muted };
        draw_text_custom(app, &part.name, 24.0, y, 15.0, c);
        y += 27.0;
    }

    let right_x = screen_width() - 310.0;
    draw_rectangle(
        right_x,
        54.0,
        310.0,
        screen_height() - 54.0,
        Color::new(
            theme_background(app.theme).r,
            theme_background(app.theme).g,
            theme_background(app.theme).b,
            0.90,
        ),
    );

    draw_text_custom(app, tr(app.language, "inspector"), right_x + 20.0, 88.0, 18.0, text);

    if let Some(index) = app.selected {
        if let Some(part) = app.parts.get(index) {
            let mut yy = 125.0;
            draw_text_custom(app, &part.name, right_x + 20.0, yy, 17.0, accent);
            yy += 32.0;

            draw_text_custom(
                app,
                &format!(
                    "{}: {:.2}, {:.2}, {:.2}",
                    tr(app.language, "position"),
                    part.position.x,
                    part.position.y,
                    part.position.z
                ),
                right_x + 20.0,
                yy,
                13.0,
                muted,
            );
            yy += 26.0;
            draw_text_custom(
                app,
                &format!(
                    "{}: {:.2}, {:.2}, {:.2}",
                    tr(app.language, "size"),
                    part.size.x,
                    part.size.y,
                    part.size.z
                ),
                right_x + 20.0,
                yy,
                13.0,
                muted,
            );
            yy += 26.0;
            draw_text_custom(
                app,
                &format!(
                    "{}: {:.2}, {:.2}, {:.2}",
                    tr(app.language, "rotation"),
                    part.rotation.x,
                    part.rotation.y,
                    part.rotation.z
                ),
                right_x + 20.0,
                yy,
                13.0,
                muted,
            );
        }
    } else {
        draw_text_custom(
            app,
            tr(app.language, "nothing_selected"),
            right_x + 20.0,
            125.0,
            14.0,
            muted,
        );
    }

    draw_text_custom(app, tr(app.language, "move"), 255.0, screen_height() - 58.0, 13.0, muted);
    draw_text_custom(app, tr(app.language, "vertical"), 255.0, screen_height() - 38.0, 13.0, muted);
    draw_text_custom(app, tr(app.language, "rotate"), 255.0, screen_height() - 18.0, 13.0, muted);

    if app.playing {
        draw_play_mode(app);
    } else {
        draw_text_custom(
            app,
            tr(app.language, "build_mode"),
            screen_width() * 0.5 - 45.0,
            82.0,
            14.0,
            muted,
        );
    }

    draw_text_custom(
        app,
        &app.status,
        255.0,
        35.0,
        13.0,
        muted,
    );
}

fn player_forward(player: &Player) -> Vec3 {
    let cp = player.pitch.cos();
    let sp = player.pitch.sin();

    vec3(
        player.yaw.sin() * cp,
        sp,
        -player.yaw.cos() * cp,
    )
    .normalize()
}

fn player_flat_forward(player: &Player) -> Vec3 {
    let f = vec3(
        player.yaw.sin(),
        0.0,
        -player.yaw.cos(),
    );

    if f.length_squared() < 0.0001 {
        vec3(0.0, 0.0, -1.0)
    } else {
        f.normalize()
    }
}

fn player_right(player: &Player) -> Vec3 {
    let f = player_flat_forward(player);
    vec3(-f.z, 0.0, f.x).normalize()
}

fn player_ground_height(app: &App, x: f32, z: f32) -> f32 {
    let mut highest: f32 = 0.0;

    for part in &app.parts {
        let half = part.size * 0.5;
        let top = part.position.y + half.y;

        if x >= part.position.x - half.x - PLAYER_RADIUS
            && x <= part.position.x + half.x + PLAYER_RADIUS
            && z >= part.position.z - half.z - PLAYER_RADIUS
            && z <= part.position.z + half.z + PLAYER_RADIUS
        {
            highest = highest.max(top);
        }
    }

    highest
}

fn mouse_delta_manual(app: &mut App) -> Vec2 {
    let current = vec2(mouse_position().0, mouse_position().1);

    if !app.mouse_initialized {
        app.last_mouse = current;
        app.mouse_initialized = true;
        return Vec2::ZERO;
    }

    let delta = current - app.last_mouse;
    app.last_mouse = current;
    delta
}

fn update_player_mouse(app: &mut App) {
    if !app.mouse_captured {
        return;
    }

    let delta = mouse_delta_manual(app);
    // Горизонтальная мышь была инвертирована. Теперь мышь вправо = камера вправо.
    app.player.yaw -= delta.x * 0.004;
    app.player.pitch = (app.player.pitch + delta.y * 0.004).clamp(-1.45, 1.45);
}

fn key_down_any(keys: &[KeyCode]) -> bool {
    keys.iter().any(|&key| is_key_down(key))
}

fn key_pressed_any(keys: &[KeyCode]) -> bool {
    keys.iter().any(|&key| is_key_pressed(key))
}

fn update_player(app: &mut App, dt: f32) {
    // Читаем клавиатуру каждый кадр напрямую.
    // В игре управление не зависит от состояния редактора или текстовых полей.
    let forward_pressed = key_down_any(&[KeyCode::W, KeyCode::Up]);
    let back_pressed = key_down_any(&[KeyCode::S, KeyCode::Down]);
    let left_pressed = key_down_any(&[KeyCode::A, KeyCode::Left]);
    let right_pressed = key_down_any(&[KeyCode::D, KeyCode::Right]);

    let run_pressed = key_down_any(&[KeyCode::LeftShift, KeyCode::RightShift]);
    let jump_pressed = key_pressed_any(&[KeyCode::Space]);

    let forward = player_flat_forward(&app.player);
    let right = player_right(&app.player);

    let mut wish = Vec3::ZERO;

    if forward_pressed {
        wish += forward;
    }
    if back_pressed {
        wish -= forward;
    }
    if left_pressed {
        wish -= right;
    }
    if right_pressed {
        wish += right;
    }

    if wish.length_squared() > 0.0001 {
        wish = wish.normalize();
    }

    let speed = if run_pressed {
        RUN_SPEED
    } else {
        WALK_SPEED
    };

    app.player.velocity.x = wish.x * speed;
    app.player.velocity.z = wish.z * speed;

    // Прыжок именно по нажатию Space.
    if app.player.grounded && jump_pressed {
        app.player.velocity.y = JUMP_SPEED;
        app.player.grounded = false;
    }

    app.player.velocity.y -= GRAVITY * dt;
    app.player.position += app.player.velocity * dt;

    let ground = player_ground_height(app, app.player.position.x, app.player.position.z);
    let feet = app.player.position.y - PLAYER_HEIGHT * 0.5;

    if feet <= ground {
        app.player.position.y = ground + PLAYER_HEIGHT * 0.5;
        app.player.velocity.y = 0.0;
        app.player.grounded = true;
    } else {
        app.player.grounded = false;
    }
}

fn player_camera(app: &App) -> Camera3D {
    let eye = app.player.position + vec3(0.0, 0.35, 0.0);
    let direction = player_forward(&app.player);

    Camera3D {
        position: eye,
        target: eye + direction,
        up: vec3(0.0, 1.0, 0.0),
        fovy: 75.0,
        aspect: None,
        projection: Projection::Perspective,
        ..Default::default()
    }
}

fn draw_play_mode(app: &mut App) {
    set_camera(&player_camera(app));

    for part in &app.parts {
        draw_cube(part.position, part.size, None, part.color);
    }

    draw_player_avatar(app);

    set_default_camera();

    let text = theme_text(app.theme);
    let muted = theme_muted(app.theme);

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        70.0,
        Color::new(0.0, 0.0, 0.0, 0.30),
    );
    draw_text_custom(app, tr(app.language, "play_mode"), 25.0, 32.0, 20.0, text);
    draw_text_custom(app, tr(app.language, "exit_play"), 25.0, 54.0, 13.0, muted);
    draw_text_custom(app, tr(app.language, "play_controls"), 25.0, 76.0, 13.0, muted);

    if !app.mouse_captured {
        centered_text(app, tr(app.language, "click_to_look"), screen_height() * 0.5, 18.0, text);
    }

}

fn draw_player_avatar(app: &App) {
    let base = app.player.position;
    let skin = builtin_skin_color(&app.account.skin_source);

    draw_cube(base + vec3(0.0, 0.72, 0.0), vec3(0.7, 0.9, 0.42), None, skin);
    draw_cube(base + vec3(0.0, 1.30, 0.0), vec3(0.72, 0.72, 0.72), None, skin);

    draw_cube(
        base + vec3(-0.50, 0.72, 0.0),
        vec3(0.28, 0.82, 0.30),
        None,
        skin,
    );
    draw_cube(
        base + vec3(0.50, 0.72, 0.0),
        vec3(0.28, 0.82, 0.30),
        None,
        skin,
    );
    draw_cube(
        base + vec3(-0.20, 0.0, 0.0),
        vec3(0.30, 0.80, 0.30),
        None,
        Color::new(0.12, 0.14, 0.18, 1.0),
    );
    draw_cube(
        base + vec3(0.20, 0.0, 0.0),
        vec3(0.30, 0.80, 0.30),
        None,
        Color::new(0.12, 0.14, 0.18, 1.0),
    );
}

fn start_play(app: &mut App) {
    app.playing = true;
    app.mouse_captured = false;
    app.mouse_initialized = false;
    // Сразу ставим игрока на поверхность Ground, чтобы Space работал с первого кадра.
    let ground = player_ground_height(app, 0.0, 5.0);
    app.player.position = vec3(0.0, ground + PLAYER_HEIGHT * 0.5, 5.0);
    app.player.velocity = Vec3::ZERO;
    app.player.grounded = true;
    app.status = tr(app.language, "play_started").to_string();
    show_mouse(true);
}

fn stop_play(app: &mut App) {
    app.playing = false;
    app.mouse_captured = false;
    app.mouse_initialized = false;
    app.status = tr(app.language, "play_stopped").to_string();
    show_mouse(true);
}

fn run_rhai_test() {
    let engine = Engine::new();

    let script = r#"
        print("================================");
        print(" Hello from CreaLink + Rhai! ");
        print(" Rhai работает внутри CreaLink!");
        print("================================");
    "#;

    match engine.eval::<()>(script) {
        Ok(_) => println!("Rhai script result: OK"),
        Err(error) => println!("Rhai error: {}", error),
    }
}

fn default_account() -> Account {
    Account {
        username: "Player".to_string(),
        id: generate_local_id(),
        skin_name: "Classic".to_string(),
        skin_source: "classic".to_string(),
    }
}

fn generate_local_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("local-{:x}", millis)
}

fn load_account() -> Account {
    let Ok(data) = fs::read_to_string(ACCOUNT_FILE) else {
        return default_account();
    };

    let mut account = default_account();

    for line in data.lines() {
        if let Some(value) = line.strip_prefix("username=") {
            account.username = value.to_string();
        } else if let Some(value) = line.strip_prefix("id=") {
            account.id = value.to_string();
        } else if let Some(value) = line.strip_prefix("skin_name=") {
            account.skin_name = value.to_string();
        } else if let Some(value) = line.strip_prefix("skin_source=") {
            account.skin_source = value.to_string();
        }
    }

    account
}

fn save_account(account: &Account) {
    let data = format!(
        "username={}\nid={}\nskin_name={}\nskin_source={}\n",
        account.username.replace('\n', ""),
        account.id.replace('\n', ""),
        account.skin_name.replace('\n', ""),
        account.skin_source.replace('\n', "")
    );

    let _ = fs::write(ACCOUNT_FILE, data);
}

fn builtin_skin_color(kind: &str) -> Color {
    match kind {
        "robot" => Color::new(0.58, 0.62, 0.68, 1.0),
        "void" => Color::new(0.20, 0.12, 0.30, 1.0),
        _ => Color::new(0.76, 0.54, 0.38, 1.0),
    }
}

async fn apply_builtin_skin(app: &mut App, kind: &str) {
    let name = match kind {
        "robot" => "Robot",
        "void" => "Void",
        _ => "Classic",
    };

    let path = Path::new(SKIN_DIR);
    let _ = fs::create_dir_all(path);

    let width = 64;
    let height = 64;
    let color = builtin_skin_color(kind);
    let rgba = [
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        255u8,
    ];

    let mut pixels = vec![0u8; width * height * 4];
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&rgba);
    }

    let texture = Texture2D::from_rgba8(width as u16, height as u16, &pixels);
    texture.set_filter(FilterMode::Nearest);

    let file = Path::new(SKIN_DIR).join(format!("{}.rgba", kind));
    let _ = fs::write(&file, &pixels);

    app.skin_texture = Some(texture);
    app.skin_path = Some(file);
    app.account.skin_name = name.to_string();
    app.account.skin_source = kind.to_string();
    save_account(&app.account);
    app.status = tr(app.language, "skin_ready").to_string();
}

async fn load_current_skin(app: &mut App) {
    let source = app.account.skin_source.clone();

    if source.starts_with("http://") || source.starts_with("https://") {
        let _ = load_skin_from_url(app, &source).await;
    } else {
        apply_builtin_skin(app, &source).await;
    }
}

async fn download_skin(app: &mut App) {
    let url = app.skin_url_edit.trim().to_string();

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        app.status = tr(app.language, "skin_error").to_string();
        return;
    }

    app.skin_loading = true;
    app.status = "Downloading...".to_string();

    match load_skin_from_url(app, &url).await {
        Ok(_) => {
            app.account.skin_source = url.clone();
            save_account(&app.account);
            app.status = tr(app.language, "skin_ready").to_string();
        }
        Err(error) => {
            eprintln!("Skin error: {error}");
            app.status = tr(app.language, "skin_error").to_string();
        }
    }

    app.skin_loading = false;
}

async fn load_skin_from_url(app: &mut App, url: &str) -> Result<(), String> {
    let response = ureq::get(url)
        .call()
        .map_err(|e| e.to_string())?;

    let bytes = response
        .into_body()
        .read_to_vec()
        .map_err(|e| e.to_string())?;

    let decoded = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .to_rgba8();

    let width = decoded.width();
    let height = decoded.height();

    if width == 0 || height == 0 || width > 2048 || height > 2048 {
        return Err("invalid image size".to_string());
    }

    let raw = decoded.into_raw();
    let texture = Texture2D::from_rgba8(width as u16, height as u16, &raw);
    texture.set_filter(FilterMode::Nearest);

    fs::create_dir_all(SKIN_DIR).map_err(|e| e.to_string())?;
    let file = Path::new(SKIN_DIR).join("internet_skin.png");
    fs::write(&file, &bytes).map_err(|e| e.to_string())?;

    app.skin_texture = Some(texture);
    app.skin_path = Some(file);
    app.account.skin_name = "Internet Skin".to_string();

    Ok(())
}
