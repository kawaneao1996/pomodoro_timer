use dioxus::prelude::*;
use gloo_timers::callback::Interval;

// ポモドーロタイマーの状態
#[derive(Debug, Clone, PartialEq)]
enum TimerMode {
    Work,
    ShortBreak,
    LongBreak,
}

// ポモドーロタイマーの設定
struct TimerSettings {
    work_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
    sessions_before_long_break: u32,
}

// デフォルト設定
impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_before_long_break: 4,
        }
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn PomodoroTimer() -> Element {
    let settings = use_signal(|| TimerSettings::default());
    let mut mode = use_signal(|| TimerMode::Work);
    let mut seconds_remaining = use_signal(|| settings.with(|s| s.work_minutes) * 60);
    let mut is_active = use_signal(|| false);
    let sessions_completed = use_signal(|| 0);
    let mut interval = use_signal(|| None::<Interval>);

    // タイマーを開始する関数
    let mut start_timer = move || {
        if !is_active() {
            is_active.set(true);

            // 既存のインターバルがあれば解除
            if interval.with(|i| i.is_some()) {
                interval.set(None);
            }

            // 新しいインターバルを作成
            let timer_callback = {
                let mut seconds_remaining = seconds_remaining.clone();
                let mut is_active = is_active.clone();
                let mut mode = mode.clone();
                let mut sessions_completed = sessions_completed.clone();
                let settings = settings.clone();
                let mut interval = interval.clone();

                move || {
                    if seconds_remaining() <= 1 {
                        // タイマー終了時の処理
                        seconds_remaining.set(0);
                        is_active.set(false);

                        // インターバルを停止
                        interval.set(None);

                        // 次のモードに切り替え
                        match mode() {
                            TimerMode::Work => {
                                let current_sessions = sessions_completed() + 1;
                                sessions_completed.set(current_sessions);

                                // 長い休憩の条件を満たしているかチェック
                                if current_sessions
                                    % settings.with(|s| s.sessions_before_long_break)
                                    == 0
                                {
                                    mode.set(TimerMode::LongBreak);
                                    seconds_remaining
                                        .set(settings.with(|s| s.long_break_minutes) * 60);
                                } else {
                                    mode.set(TimerMode::ShortBreak);
                                    seconds_remaining
                                        .set(settings.with(|s| s.short_break_minutes) * 60);
                                }
                            }
                            TimerMode::ShortBreak | TimerMode::LongBreak => {
                                mode.set(TimerMode::Work);
                                seconds_remaining.set(settings.with(|s| s.work_minutes) * 60);
                            }
                        }
                    } else {
                        // カウントダウン
                        seconds_remaining.set(seconds_remaining() - 1);
                    }
                }
            };

            // 1秒ごとにタイマーを更新
            let new_interval = Interval::new(1000, timer_callback);
            interval.set(Some(new_interval));
        }
    };

    // タイマーを一時停止する関数
    let mut pause_timer = move || {
        if is_active() {
            is_active.set(false);
            interval.set(None);
        }
    };

    // タイマーをリセットする関数
    let mut reset_timer = move || {
        is_active.set(false);
        interval.set(None);

        // 現在のモードに応じた時間にリセット
        match mode() {
            TimerMode::Work => seconds_remaining.set(settings.with(|s| s.work_minutes) * 60),
            TimerMode::ShortBreak => {
                seconds_remaining.set(settings.with(|s| s.short_break_minutes) * 60)
            }
            TimerMode::LongBreak => {
                seconds_remaining.set(settings.with(|s| s.long_break_minutes) * 60)
            }
        }
    };

    // モードを変更する関数
    let mut change_mode = move |new_mode: TimerMode| {
        if mode() != new_mode {
            mode.set(new_mode.clone());
            is_active.set(false);
            interval.set(None);

            // 新しいモードに応じた時間を設定
            match new_mode {
                TimerMode::Work => seconds_remaining.set(settings.with(|s| s.work_minutes) * 60),
                TimerMode::ShortBreak => {
                    seconds_remaining.set(settings.with(|s| s.short_break_minutes) * 60)
                }
                TimerMode::LongBreak => {
                    seconds_remaining.set(settings.with(|s| s.long_break_minutes) * 60)
                }
            }
        }
    };

    // 残り時間を表示形式に変換
    let minutes = seconds_remaining() / 60;
    let seconds = seconds_remaining() % 60;
    let time_display = format!("{:02}:{:02}", minutes, seconds);

    // モードに応じたラベルを取得
    let mode_label = match mode() {
        TimerMode::Work => "仕事",
        TimerMode::ShortBreak => "短い休憩",
        TimerMode::LongBreak => "長い休憩",
    };

    rsx! {
        div { class: "pomodoro-container",
            h2 { "ポモドーロタイマー" }

            // モード選択ボタン
            div { class: "mode-buttons",
                button {
                    class: if mode() == TimerMode::Work { "mode-button active" } else { "mode-button" },
                    onclick: move |_| change_mode(TimerMode::Work),
                    "仕事"
                }
                button {
                    class: if mode() == TimerMode::ShortBreak { "mode-button active" } else { "mode-button" },
                    onclick: move |_| change_mode(TimerMode::ShortBreak),
                    "短い休憩"
                }
                button {
                    class: if mode() == TimerMode::LongBreak { "mode-button active" } else { "mode-button" },
                    onclick: move |_| change_mode(TimerMode::LongBreak),
                    "長い休憩"
                }
            }

            // タイマー表示
            div { class: "timer-display", "{time_display}" }

            // 現在のモード表示
            p { class: "current-mode", "現在のモード: {mode_label}" }

            // タイマー制御ボタン
            div { class: "timer-controls",
                button {
                    class: "timer-button start-button",
                    disabled: is_active(),
                    onclick: move |_| start_timer(),
                    "開始"
                }
                button {
                    class: "timer-button pause-button",
                    disabled: !is_active(),
                    onclick: move |_| pause_timer(),
                    "一時停止"
                }
                button {
                    class: "timer-button reset-button",
                    onclick: move |_| reset_timer(),
                    "リセット"
                }
            }

            // セッション情報
            div { class: "session-info",
                p { "完了したセッション: {sessions_completed}" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        PomodoroTimer {}
    }
}

#[component]
fn Navbar() -> Element {
    rsx! {
        div { id: "navbar",
            Link { to: Route::Home {}, "ホーム" }
            a {
                href: "https://kawaneao1996.github.io/myblogs/",
                target: "_blank",
                "ブログ"
            }
        }

        Outlet::<Route> {}
    }
}
