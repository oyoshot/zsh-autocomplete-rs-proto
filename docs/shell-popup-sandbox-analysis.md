# shell-popup-sandbox 完全分析

リファレンス: `tmp/shell-popup-sandbox/`

## 概要

テキストエディタ風のポップアップ補完UI のプロトタイプ。Rust バイナリ + Zsh スクリプトの構成で、zsh-autocomplete-rs の基盤となった。

## Rust 側 (5ファイル)

### src/main.rs (150行)

#### CLI 引数解析
手動で `std::env::args()` をパースする:
```
--prefix <text>        : 補完対象のプリフィックス
--cursor-row <n>       : カーソル行 (0-indexed)
--cursor-col <n>       : カーソル列 (0-indexed)
--candidates <text>    : 改行区切りの候補テキスト
```

**問題点**: `--candidates` は CLI 引数として渡されるため、ARG_MAX 制限に掛かる可能性がある。

#### TTY 管理
```rust
let mut tty = File::options().read(true).write(true).open("/dev/tty")?;
terminal::enable_raw_mode()?;
```
- `/dev/tty` を read/write で open
- stdout はシェルの `$(...)` にキャプチャされるため、UI 描画は tty に直接出力

#### Panic フック
```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let _ = terminal::disable_raw_mode();
    original_hook(info);
}));
```
panic 時に raw mode を復元する。本プロジェクトでは TtyGuard (Drop) に置換。

#### イベントループ
```rust
let exit_code = loop {
    match input::read_action()? {
        Action::MoveDown => { app.move_down(); ui::draw(...)?; }
        Action::MoveUp   => { app.move_up();   ui::draw(...)?; }
        Action::Confirm  => { ui::clear(...)?; print!("{}", selected); break 0; }
        Action::DismissWithSpace => { ui::clear(...)?; print!("{} ", filter_text); break 2; }
        Action::Cancel   => { ui::clear(...)?; print!("{}", filter_text_if_changed); break 1; }
        Action::TypeChar(c) => { ui::clear(...)?; app.type_char(c); ... }
        Action::Backspace   => { ui::clear(...)?; app.backspace(); ... }
        Action::None => {}
    }
};
```

TypeChar/Backspace で `filtered` が空になった場合、自動的に exit code 1 で終了する。

#### 終了コード仕様

| コード | トリガー | stdout 出力 |
|--------|---------|-------------|
| 0 | Tab/Enter (Confirm) | 選択された候補テキスト |
| 1 | Esc/^C (Cancel) | filter_text (prefix と異なる場合) or 空 |
| 1 | filtered が空になった場合 | filter_text |
| 2 | Space (DismissWithSpace) | filter_text + スペース |

### src/app.rs (96行)

#### フィールド

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `all_candidates` | `Vec<String>` | 全候補 (不変) |
| `filtered` | `Vec<String>` | フィルタ後の候補 |
| `filter_text` | `String` | ユーザーが入力したフィルタテキスト (prefix から開始) |
| `selected` | `usize` | 選択中のインデックス |
| `scroll_offset` | `usize` | スクロール位置 |
| `max_visible` | `usize` | 最大表示行数 (**10**) |
| `cursor_row` | `u16` | カーソル行 |
| `cursor_col` | `u16` | カーソル列 |
| `prefix` | `String` | 元のプリフィックス |
| `fuzzy` | `FuzzyMatcher` | ファジーマッチャー |

#### フィルタ更新

```rust
pub fn update_filter(&mut self) {
    let scored = self.fuzzy.filter(&self.all_candidates, &self.filter_text);
    self.filtered = scored.into_iter().map(|s| s.text).collect();
    self.selected = 0;
    self.scroll_offset = 0;
}
```

フィルタ更新時に選択とスクロールをリセットする。

#### スクロールロジック

**move_down** (循環あり):
```
selected + 1 < len → selected++
selected + 1 == len → selected = 0, scroll_offset = 0  (先頭に戻る)
selected >= scroll_offset + max_visible → scroll_offset = selected + 1 - max_visible
```

**move_up** (循環あり):
```
selected > 0 → selected--
selected == 0 → selected = len - 1, scroll_offset = len - max_visible  (末尾に飛ぶ)
selected < scroll_offset → scroll_offset = selected
```

### src/ui.rs (192行)

#### Popup 位置計算

```rust
pub struct Popup { pub row: u16, pub col: u16, pub width: u16, pub height: u16 }
```

**定数**:
- `MAX_POPUP_WIDTH = 60`
- `PADDING = 2`

**位置決定ロジック**:
```
width = min(max_content_width + PADDING + 2, MAX_POPUP_WIDTH, term_cols)
height = num_visible + 2  (上下ボーダー)

col = min(cursor_col, term_cols - width)  // 右端に収める

space_below = term_rows - cursor_row - 1
row = if space_below >= height { cursor_row + 1 }  // 下に表示
      else { cursor_row - height }                   // 上に表示
```

`max_content_width` はフィルタラベル (`" filter "`) と全候補の幅の最大値。

#### ボックス描画

```
┌ filter_text ─────────┐   ← 上部ボーダー + フィルタテキスト
│  candidate1          │   ← 通常行
│▶ candidate2          │   ← 選択行 (Reverse attribute)
│  candidate3          │
└──────────────────────┘   ← 下部ボーダー
```

- 選択行: `SetAttribute(Attribute::Reverse)` でハイライト
- パディング: 候補テキストの右側をスペースで埋め

#### filter_text インライン表示

プロンプト行にフィルタテキストを上書き表示:
```rust
let prefix_start_col = cursor_col - prefix_width;
crossterm::execute!(tty,
    cursor::MoveTo(prefix_start_col, cursor_row),
    Print(filter_display),
);
```

prefix より短くなった場合はスペースでクリア。

#### clear 関数

1. ポップアップ領域をスペースで埋め
2. プロンプト行に元の prefix を復元
3. カーソル位置を復元

#### truncate_to_width

```rust
fn truncate_to_width(s: &str, max_width: usize) -> String {
    // Unicode 文字幅を考慮して切り詰め
    // 幅超過時は '…' を付加
}
```

`unicode_width::UnicodeWidthChar` を使用。全角文字は幅2として計算。

### src/input.rs (38行)

#### キーマップ表

| キー | Action | 説明 |
|------|--------|------|
| `Tab` | `Confirm` | 選択確定 |
| `Shift+Tab` | `MoveUp` | 上に移動 |
| `Space` | `DismissWithSpace` | テキスト確定 + スペース + 抑制 |
| `Down` | `MoveDown` | 下に移動 |
| `Up` | `MoveUp` | 上に移動 |
| `Enter` | `Confirm` | 選択確定 |
| `Esc` | `Cancel` | キャンセル |
| `Ctrl+C` | `Cancel` | キャンセル |
| `Backspace` | `Backspace` | フィルタ1文字削除 |
| `Char(c)` | `TypeChar(c)` | フィルタに1文字追加 |
| その他 | `None` | 無視 |

#### Poll タイムアウト

```rust
if !event::poll(Duration::from_millis(100))? {
    return Ok(Action::None);
}
```

100ms のポーリング間隔。タイムアウト時は `None` を返す。

### src/fuzzy.rs (81行)

#### nucleo 使用法

```rust
let pattern = Pattern::new(
    query,
    CaseMatching::Smart,      // 小文字→無視、大文字→区別
    Normalization::Smart,
    AtomKind::Fuzzy,           // ファジーマッチ
);
pattern.score(haystack, &mut self.matcher)  // Option<u32>
```

#### 空クエリ時の逆順スコア

```rust
if query.is_empty() {
    return candidates.iter().enumerate().map(|(i, c)| ScoredCandidate {
        text: c.clone(),
        score: (candidates.len() - i) as u32,  // 逆順: 先頭が高スコア
    }).collect();
}
```

空クエリでは全候補を元の順序で返す。

#### テスト

| テスト | 内容 |
|--------|------|
| `empty_query_returns_all` | 空クエリで全候補が返る |
| `fuzzy_match_filters` | "foo" で "foo", "foobar" がマッチ |
| `no_match_returns_empty` | "zzz" でマッチなし |

## Zsh 側 (shell-popup.zsh, 180行)

### CPR (Cursor Position Report)

```zsh
_shell_popup_get_cursor_pos() {
    echo -ne '\e[6n' > /dev/tty          # Device Status Report 送信
    IFS='' read -t 1 -rs -d R pos < /dev/tty  # 応答読み取り
    pos="${pos#*\[}"                       # ESC[ を除去
    cursor_row=$(( ${pos%;*} - 1 ))       # row (1-indexed → 0-indexed)
    cursor_col=$(( ${pos#*;} - 1 ))       # col (1-indexed → 0-indexed)
}
```

**エッジケース**: `read -t 1` で 1 秒のタイムアウト。ターミナルが CPR に対応しない場合は失敗する。

### 候補収集

```zsh
_shell_popup_gather_candidates() {
    local prefix="$1"
    file_candidates=( ${prefix}*(N) )                  # glob (N=nullglob)
    cmd_candidates=( ${(k)commands[(I)${prefix}*]} )   # commands hash
    all_candidates=( "${(@u)file_candidates[@]}" "${(@u)cmd_candidates[@]}" )  # 重複除去
    all_candidates=( ${all_candidates:#} )              # 空文字除去
    echo "${(pj:\n:)all_candidates}"                    # 改行区切り出力
}
```

**制限**: description なし。サブコマンドやオプション補完はできない。

### バイナリ呼び出しと結果処理

```zsh
output=$("$SHELL_POPUP_BIN" \
    --prefix "$prefix" \
    --cursor-row "$cursor_row" \
    --cursor-col "$cursor_col" \
    --candidates "$candidates_str" </dev/tty)
local exit_code=$?
```

- `</dev/tty` リダイレクトで TTY 入力を確保
- `$(...)` で stdout をキャプチャ

### Suppression フラグ機構

| 状態 | `_SHELL_POPUP_SUPPRESSED` | 条件 |
|------|---------------------------|------|
| 有効 | 1 | DismissWithSpace (exit code 2) 後 |
| 解除 | 0 | スペース入力時、prefix が空になった時、Tab 実行時、Confirm/Cancel 時 |

自動トリガーの self-insert 内で:
```zsh
if [[ "$KEYS" == " " ]]; then
    _SHELL_POPUP_SUPPRESSED=0  # スペースで解除
    return
fi
(( _SHELL_POPUP_SUPPRESSED )) && return  # 抑制中はスキップ
```

### ウィジェット定義

| ウィジェット | 役割 |
|-------------|------|
| `shell-popup-self-insert` | 文字挿入 + 自動トリガー |
| `shell-popup-backward-delete-char` | Backspace + 抑制解除 |
| `shell-popup-tab-complete` | Tab 補完 (明示トリガー) |
| `shell-popup-backward-char` | ← + 抑制解除 |
| `shell-popup-forward-char` | → + 抑制解除 |

### Keybinding

```zsh
bindkey '^I' shell-popup-tab-complete
bindkey '^?' shell-popup-backward-delete-char
bindkey '\e[D' shell-popup-backward-char
bindkey '\e[C' shell-popup-forward-char

# 全 ASCII printable (32-126) を rebind
for i in {32..126}; do
    bindkey "$char" shell-popup-self-insert
done
```

**最大の副作用**: 95 個のキーバインドを上書きする。

## 定数・設定値まとめ

| 定数 | 値 | 場所 |
|------|-----|------|
| `MAX_POPUP_WIDTH` | 60 | `ui.rs` |
| `PADDING` | 2 | `ui.rs` |
| `max_visible` | 10 | `app.rs` |
| Poll timeout | 100ms | `input.rs` |
| CPR timeout | 1s | `shell-popup.zsh` |
| 自動トリガー最小入力長 | 2 | `shell-popup.zsh` |
| 自動トリガー最小候補数 | 2 | `shell-popup.zsh` |

## Cargo.toml

```toml
[package]
name = "shell-popup-windos"
version = "0.1.0"
edition = "2024"          # ← typo: "2024" は存在しない

[dependencies]
crossterm = "0.28"
nucleo-matcher = "0.3"
unicode-width = "0.2"
```

## エッジケース・バグ

### 1. edition typo
`edition = "2024"` は Rust に存在しない edition。正しくは `"2021"` (最新安定版は 2024 年時点で 2021)。
ビルドは通るが、将来の Rust バージョンで問題になる可能性。

### 2. CPR timeout
`read -t 1` で 1 秒待つ。応答がない場合 `cursor_row=0, cursor_col=0` になり、ポップアップが画面左上に表示される。

### 3. glob 特殊文字
prefix に `*`, `?`, `[` 等の glob 特殊文字が含まれると、`${prefix}*(N)` が意図しない展開をする可能性。

### 4. commands hash 更新
`commands` 連想配列は `rehash` またはシェル起動時に生成される。新しくインストールしたコマンドは即座には反映されない。

### 5. --candidates の ARG_MAX
大量の候補を `--candidates` CLI 引数で渡すと、OS の引数長制限 (Linux: ~2MB) に到達する。本プロジェクトでは stdin に変更して解決。
