# 実現方法ガイド

## 全体アーキテクチャ

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Zsh プラグイン                                │
│                                                                     │
│  source plugin.zsh                                                  │
│  ├── _zacrs_util.zsh   (CPR, prefix抽出)                            │
│  ├── _zacrs_gather.zsh (候補収集)                                    │
│  │                                                                  │
│  ├── [Tab] → _zacrs_tab_complete widget                             │
│  │   ├── prefix 抽出                                                │
│  │   ├── 候補収集 → candidates                                      │
│  │   ├── CPR でカーソル座標取得                                      │
│  │   └── Rust バイナリ呼び出し ──────────────────────┐               │
│  │                                                   │               │
│  ├── [line-pre-redraw] → _zacrs_line_pre_redraw      │               │
│  │   ├── LBUFFER 変更検知                            │               │
│  │   ├── suppression チェック                         │               │
│  │   ├── 最小入力長チェック (2文字)                    │               │
│  │   └── 候補2以上なら Rust バイナリ呼び出し ────────┤               │
│  │                                                   │               │
│  │   ┌───────────────────────────────────────────────┘               │
│  │   ▼                                                               │
│  │   candidates | zsh-autocomplete-rs complete \                     │
│  │                 --prefix "$prefix" \                               │
│  │                 --cursor-row $row --cursor-col $col               │
│  │                 </dev/tty                                         │
│  │                                                                   │
│  │   exit code + stdout で結果受信                                   │
│  │   ├── 0: 確定 → LBUFFER 置換                                     │
│  │   ├── 1: キャンセル → 変更なし or filter_text 反映                │
│  │   └── 2: dismiss+space → LBUFFER 置換 + suppress ON              │
│  │                                                                   │
│  └── LBUFFER 更新                                                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                     Rust バイナリ                                     │
│                                                                     │
│  stdin ──→ 候補読み込み (text\tdescription 形式)                     │
│                                                                     │
│  /dev/tty ──→ UI 描画 (crossterm)                                    │
│  │  ┌─────────────────────┐                                         │
│  │  │ ┌ filter ─────────┐ │ ← ボックス描画                          │
│  │  │ │▶ candidate1     │ │ ← 選択行 (Reverse)                      │
│  │  │ │  candidate2     │ │                                         │
│  │  │ │  candidate3     │ │ ← description は DarkGrey               │
│  │  │ └─────────────────┘ │                                         │
│  │  └─────────────────────┘                                         │
│  │                                                                   │
│  /dev/tty ←── キー入力 (crossterm event, poll 100ms)                 │
│  │  Tab/Enter → Confirm                                              │
│  │  Shift+Tab → MoveUp                                               │
│  │  Space     → DismissWithSpace                                     │
│  │  Esc/^C   → Cancel                                                │
│  │  文字     → TypeChar → filter 更新 → 再描画                       │
│  │  Backspace → filter 削除 → 再描画                                 │
│  │                                                                   │
│  stdout ──→ 選択テキスト出力                                         │
│  exit code ──→ 0=確定, 1=キャンセル, 2=dismiss+space                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Zsh 側の実装方法

### 候補収集 (`_zacrs_gather.zsh`)

現在の Phase 1 実装:

```zsh
# ファイル候補: glob
file_candidates=( ${prefix}*(N) )
# → ディレクトリなら "/" 追加、description "directory"/"file"

# コマンド候補 (先頭ワードの場合のみ):
cmd_candidates=(
    ${(k)commands[(I)${prefix}*]}   # 外部コマンド
    ${(k)aliases[(I)${prefix}*]}    # エイリアス
    ${(k)builtins[(I)${prefix}*]}   # ビルトイン
    ${(k)functions[(I)${prefix}*]}  # 関数 (_* を除外)
)

# 出力: text\tdescription\n
```

将来 (Phase 3+) は **zpty + compadd override** 方式に移行:

```zsh
# zpty 内で compadd を関数として再定義
compadd() {
    # -d (display), -X (explanation) 等の引数から text/description を抽出
    # 結果を stdout に text\tdescription 形式で出力
}

# zpty 内で _main_complete を実行
# → compadd が呼ばれるたびに候補をキャプチャ
```

### 自動トリガー (`line-pre-redraw` フック)

sandbox の **self-insert rebind** を置換:

```zsh
# sandbox: 全 ASCII キーを rebind (副作用大)
for i in {32..126}; do bindkey "$char" shell-popup-self-insert; done

# 本プロジェクト: ZLE フック (副作用なし)
add-zle-hook-widget line-pre-redraw _zacrs_line_pre_redraw
```

`_zacrs_line_pre_redraw` の動作:
1. `LBUFFER` が前回から変更されたか検知 (`_zacrs_prev_lbuffer`)
2. 変更なし or 抑制中 → 何もしない
3. prefix 長が最小値 (デフォルト2) 未満 → 何もしない
4. 候補数が 2 未満 → 何もしない
5. 条件を満たしたら `_zacrs_invoke` でポップアップ表示

### Tab 補完 (`^I` keybinding)

```zsh
bindkey '^I' _zacrs_tab_complete  # Tab のみ上書き
```

`_zacrs_tab_complete` の動作:
1. prefix がなければ `expand-or-complete` にフォールバック
2. 候補収集
3. 候補なし → `expand-or-complete` にフォールバック
4. 候補 1 個 → 即座に LBUFFER 挿入
5. 候補 2 個以上 → `_zacrs_invoke` でポップアップ表示

### 結果反映

```zsh
# LBUFFER の prefix 部分を選択テキストで置換
LBUFFER="${LBUFFER%$prefix}${output}"
```

exit code による分岐:
- **0 (確定)**: 選択候補で prefix を置換
- **1 (キャンセル)**: output があれば filter_text で置換、なければ変更なし
- **2 (dismiss+space)**: output + スペースで置換、suppress ON

### CPR によるカーソル座標取得

```zsh
_zacrs_get_cursor_pos() {
    echo -ne '\e[6n' > /dev/tty     # Device Status Report 送信
    IFS='' read -t 1 -rs -d R pos < /dev/tty  # 応答読み取り (timeout 1s)
    pos="${pos#*\[}"                  # ESC[ を除去
    cursor_row=$(( ${pos%;*} - 1 ))  # row (0-indexed)
    cursor_col=$(( ${pos#*;} - 1 ))  # col (0-indexed)
}
```

## Rust 側の実装方法

### 候補受信 (stdin)

```
text1\tdescription1\n
text2\tdescription2\n
text3\n                  ← description なし
```

- `\t` (tab) 区切りで text と description を分離
- ARG_MAX を回避するため、`--candidates` 引数ではなく stdin を使用
- `Candidate { text: String, description: Option<String> }`

### UI 描画 (`/dev/tty` + crossterm)

stdin は候補データで使用済み、stdout はシェルが `$(...)` でキャプチャ。
そのため、UI は `/dev/tty` に直接描画する。

```rust
let mut tty = File::options().read(true).write(true).open("/dev/tty")?;
```

描画フロー:
1. `Popup::compute()` でポップアップ位置を計算
   - カーソル下に十分な空間 → 下に表示
   - そうでなければ上に表示
   - 右端調整: `col.min(term_cols - width)`
   - 定数: `MAX_POPUP_WIDTH=60`, `PADDING=2`, `max_visible=10`
2. ボックス描画: `┌─┐` / `│ │` / `└─┘`
3. フィルタテキストを上部ボーダーに表示: `┌ filter ─────┐`
4. 選択行を `SetAttribute(Reverse)` でハイライト
5. description を `DarkGrey` で右寄せ表示
6. プロンプト行に filter_text をインライン表示

### ファジーフィルタ (nucleo-matcher)

```rust
let pattern = Pattern::new(query, CaseMatching::Smart, Normalization::Smart, AtomKind::Fuzzy);
```

- 空クエリ: 全候補を元の順序で返却 (逆順スコアで安定ソート)
- クエリあり: スコア降順でフィルタ
- `CaseMatching::Smart`: 小文字 → 大文字小文字無視、大文字含む → 大文字小文字区別

### 終了コード

| コード | 意味 | stdout | Zsh 側の処理 |
|--------|------|--------|-------------|
| 0 | 確定 (Tab/Enter) | 選択候補テキスト | LBUFFER 置換 |
| 1 | キャンセル (Esc/^C) | filter_text (変更ありの場合) or 空 | 条件付き LBUFFER 置換 |
| 2 | dismiss+space (Space) | filter_text | LBUFFER 置換 + suppress ON |

### Panic 安全性 (TtyGuard)

```rust
pub struct TtyGuard {
    tty: File,
}

impl TtyGuard {
    pub fn new() -> io::Result<Self> {
        let tty = File::options().read(true).write(true).open("/dev/tty")?;
        terminal::enable_raw_mode()?;
        Ok(Self { tty })
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
```

- `Drop` トレイトで raw mode を確実に復元
- panic 発生時も `drop()` が呼ばれるため安全
- sandbox では手動で panic hook を設定していたが、TtyGuard で自動化

## sandbox からの変更点

### データ構造の変更

| sandbox | 本プロジェクト | 理由 |
|---------|--------------|------|
| `Vec<String>` | `Vec<Candidate>` | description を持たせるため |
| `--candidates` CLI 引数 | stdin パイプ | ARG_MAX 回避 (大量候補対応) |
| 手動引数解析 | clap derive | 保守性向上 |

### 候補収集の変更

| sandbox | 本プロジェクト | 理由 |
|---------|--------------|------|
| glob + commands hash | glob + commands/aliases/builtins/functions | より多くの候補ソース |
| description なし | description あり (directory/file/command 等) | UI の情報量向上 |

### 自動トリガーの変更

| sandbox | 本プロジェクト | 理由 |
|---------|--------------|------|
| 全 ASCII rebind (95 keys) | `line-pre-redraw` ZLE フック | 副作用の劇的削減 |
| `shell-popup-self-insert` widget | `_zacrs_line_pre_redraw` hook | キーバインドへの介入なし |

### UI の分割

| sandbox | 本プロジェクト | 理由 |
|---------|--------------|------|
| `ui.rs` (1ファイル) | `ui/popup.rs` + `ui/render.rs` + `ui/theme.rs` | 責務の分離 |
| 色ハードコード | `theme.rs` に集約 | 将来のテーマカスタマイズ |
| description 非対応 | description 列表示 (DarkGrey) | 候補の情報量向上 |

### その他

| sandbox | 本プロジェクト | 理由 |
|---------|--------------|------|
| `edition = "2024"` (typo) | `edition = "2024"` (要修正→2021) | Rust 2024 edition は未リリース |
| panic hook 手動設定 | TtyGuard (Drop) | より安全な復元保証 |
| 1ファイル `shell-popup.zsh` | 3ファイルに分割 | 責務の分離 |

## 技術的な課題と解決策

### ARG_MAX 問題

**問題**: 大量の候補を `--candidates` CLI 引数で渡すと、OS の ARG_MAX 制限 (Linux: ~2MB) に到達する可能性がある。

**解決策**: stdin パイプで候補を渡す。

```zsh
# sandbox (ARG_MAX に制限される):
$BIN --candidates "$candidates_str" </dev/tty

# 本プロジェクト (制限なし):
echo "$candidates_str" | $BIN complete --prefix "$prefix" ... </dev/tty
```

注意: stdin を候補データに使うため、`</dev/tty` リダイレクトで TTY 入力を確保。

### ターミナル制御の競合

**問題**: Rust バイナリが raw mode でターミナルを制御するが、シェルも同じターミナルを使用。

**解決策**:
- UI 描画は `/dev/tty` に直接出力 (stdout と分離)
- シェルの `$(...)` は stdout のみキャプチャするため、UI 描画と干渉しない
- raw mode は Rust バイナリ内で完結。終了時に確実に復元 (TtyGuard)

### 非同期補完のタイミング (将来)

**問題**: `line-pre-redraw` は入力のたびに発火する。毎回候補収集すると遅い。

**解決策** (Phase 3):
- zsh-autocomplete と同様に遅延 (delay) を導入: デフォルト 0.05s
- zpty 内でバックグラウンド補完
- FD callback で結果を受信

### 他プラグインとの共存

**問題**: zsh-autosuggestions, fast-syntax-highlighting 等と衝突しないか。

**解決策**:
- keybinding は `^I` のみ → 他プラグインのバインドと衝突しない
- ZLE フック (`line-pre-redraw`) は複数登録可能 → 共存可能
- グローバル変数を変更しない → 他プラグインの設定を壊さない
- compinit に触れない → 他プラグインの補完設定に影響しない
