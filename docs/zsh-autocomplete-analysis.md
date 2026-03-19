# zsh-autocomplete 完全分析

リファレンス: `tmp/zsh-autocomplete/`

## 初期化フロー

```
plugin.zsh
├── unsetopt listbeep
├── zmodload zsh/parameter, zsh/param/private
├── _autocomplete__func_opts 定義 (グローバルオプションセット)
├── FPATH に Completions/ を追加
├── autoload Functions/**/.autocomplete__*
└── .autocomplete__main
    ├── zmodload (zsh/files, zsh/parameter, zsh/zleparameter, zsh/zutil)
    ├── autoload add-zsh-hook, zmathfunc
    ├── ディレクトリ作成 ($XDG_DATA_HOME/zsh, ログ, キャッシュ)
    ├── 古いログのクリーンアップ (7日以上)
    ├── 6モジュールの順次読み込み:
    │   ├── .autocomplete__compinit   → compdef ラッパー、bashcompinit 遅延ロード
    │   ├── .autocomplete__config     → zstyle 設定、ZLE_*_SUFFIX_CHARS
    │   ├── .autocomplete__widgets    → ZLE/Completion ウィジェット登録
    │   ├── .autocomplete__key-bindings → 4 keymap のバインド
    │   ├── .autocomplete__recent-dirs  → (precmd で初期化)
    │   └── .autocomplete__async      → (precmd で初期化)
    ├── add-zsh-hook precmd .autocomplete__main:precmd
    └── precmd_functions の先頭に挿入
```

precmd (初回のみ実行):
```
.autocomplete__main:precmd
├── zsh-syntax-highlighting との互換性修正
├── 各モジュールの :precmd を実行:
│   ├── .autocomplete__compinit:precmd   → compinit 実行、_main_complete パッチ
│   ├── .autocomplete__config:precmd     → menu/list-prompt 削除
│   ├── .autocomplete__widgets:precmd    → completion widget 作成
│   ├── .autocomplete__key-bindings:precmd → (なし)
│   ├── .autocomplete__recent-dirs:precmd → autopushd 設定、chpwd フック
│   └── .autocomplete__async:precmd     → ZLE フック登録、FD widget 作成
└── 各 :precmd を unfunction
```

## Functions/Init/ 全7ファイル

### .autocomplete__main (95行)

**目的**: エントリポイント。モジュールの順次読み込みと precmd フック登録。

**グローバル状態変更**:
- `_autocomplete__ctxt_opts` (配列): completealiases, completeinword
- `_autocomplete__mods` (配列): モジュール名リスト
- `FPATH` / `fpath`: Completions/ を先頭に追加
- `_autocomplete__log`: ログファイルパス
- `_autocomplete__ps4`: デバッグ用 PS4

**副作用**:
- ディレクトリ作成: `$XDG_DATA_HOME/zsh/`, ログディレクトリ, キャッシュディレクトリ
- 古いログ削除 (7日以上)
- 旧ログディレクトリ削除
- named directory 登録: `~autocomplete`, `~zsh-autocomplete`, `~autocomplete-log`

### .autocomplete__config (159行)

**目的**: zstyle 補完設定の一括設定。

**グローバル状態変更**:
- `ZLE_REMOVE_SUFFIX_CHARS = $' /;\n\r\t'`
- `ZLE_SPACE_SUFFIX_CHARS = '|&<>-+'`

**zstyle 設定 (主要なもの)**:
- `use-cache yes` + `cache-path` (動的)
- `completer`: `_expand _complete _complete:-fuzzy _correct _approximate _ignored`
- `max-errors`: 動的計算 `min(2, (PREFIX+SUFFIX)/3)`
- `matcher-list`: 大文字小文字、ドット前一致、ファジー用の3段階
- `prefix-needed yes`
- `ignored-patterns`: functions (`*.*`, `*:*`, `+*`), users (`_*`), widgets
- `tag-order`: command, tilde, approximate, cd, fc, git (動的)
- `file-patterns`: ディレクトリ優先、実行ファイル
- `format`: dim + bold スタイル
- `group-name`, `group-order`: グループ化設定

**precmd での追加処理**:
- `_comp_setup` に `globdots` 対応を追加
- `menu` と `list-prompt` の zstyle を強制削除
- `LISTPROMPT` を unset

### .autocomplete__compinit (189行)

**目的**: compinit の制御、`_main_complete` 等のパッチ。

**グローバル状態変更**:
- `_autocomplete__compdef` (配列): compdef 呼び出しの保存
- `compdef()` を一時ラッパーに置換
- `_bash_complete`, `compgen`, `complete` を遅延ロード

**precmd での処理**:
1. CDPATH クリア: `[[ -v CDPATH && -z $CDPATH ]] && unset CDPATH cdpath`
2. compinit 条件付き実行:
   - comp dump ファイルが古い/存在しない場合に再実行
   - `bindkey` を一時的に空関数にして keybinding 登録を抑制
3. `compinit()` を空関数に上書き (他プラグインによる再実行を防止)
4. 保存した compdef を実行
5. comp cache の zcompile (バックグラウンド)

**パッチ**:
- `_main_complete` → `autocomplete:_main_complete:new`:
  - `compstate[insert]=automenu-unambiguous`
  - `compstate[last_prompt]=yes`
  - `compstate[list]='list force packed'`
  - `TRAPINT`/`TRAPQUIT` を再定義
  - `comppostfuncs` に `_autocomplete__unambiguous`, `compstate[list_max]=0`, `MENUSCROLL=0`
- `_complete` → `autocomplete:_complete:old`:
  - `PREFIX=$PREFIX$SUFFIX; SUFFIX=`
  - 戻り値を `compstate[nmatches] > nmatches` に修正
- `_approximate` → `autocomplete:_approximate:old`:
  - compadd 関数の一時差し替えでワークアラウンド

### .autocomplete__key-bindings (65行)

**目的**: 4つの keymap へのキーバインド登録。

**バインド登録** (詳細は [side-effects-comparison.md](./side-effects-comparison.md#3-keybinding-大量上書き) 参照):
- `main`: Tab, Shift+Tab, Up, Down, Alt+Up, Alt+Down
- `emacs`: ^P, ^N, \ep, \en, ^R, ^S, ^X/
- `vicmd`: k, j, ^P, ^N, /, ?
- `menuselect`: Tab, Shift+Tab, ^@, \ev, ^_, \eu, PageUp, PageDown, 方向キー, ^R, ^S

### .autocomplete__widgets (47行)

**目的**: ZLE ウィジェットと Completion ウィジェットの登録。

**ZLE ウィジェット**:
- `up-line-or-search` → `.autocomplete__up-line-or-search__zle-widget`
- `down-line-or-select` → `.autocomplete__down-line-or-select__zle-widget`
- `history-search-backward` → `.autocomplete__history-search__zle-widget` (未定義?)

**Completion ウィジェット** (precmd で作成):
- `complete-word`, `menu-complete`, `menu-select` → `.autocomplete__complete-word__completion-widget`
- `reverse-menu-complete` → 同上
- `insert-unambiguous-or-complete` → 同上
- `menu-search` (menu-select) → 同上
- `history-search-backward` (menu-select) → `.autocomplete__history-search__completion-widget`

**Autosuggest 対応**:
- `ZSH_AUTOSUGGEST_MANUAL_REBIND=1`
- `ZSH_AUTOSUGGEST_ORIGINAL_WIDGET_PREFIX=.autosuggest-orig-`
- completion widget を `ZSH_AUTOSUGGEST_IGNORE_WIDGETS` に追加

### .autocomplete__recent-dirs (21行)

**目的**: 最近ディレクトリの追跡設定。

**グローバルオプション変更**:
- `setopt autopushd pushdignoredups` (グローバル!)

**precmd での処理**:
- `chpwd_recent_dirs`, `chpwd_recent_filehandler` を autoload
- zstyle `recent-dirs-file` 設定 (デフォルト: `$XDG_DATA_HOME/zsh/chpwd-recent-dirs`)
- zstyle `recent-dirs-max` 設定 (デフォルト: 0 = 無制限)
- `dirstack` を最近ディレクトリで初期化
- `add-zsh-hook chpwd chpwd_recent_dirs`

### .autocomplete__async (695行)

**目的**: 非同期補完の中核。zpty + FD callback によるバックグラウンド補完。

**precmd での処理**:
- ZLE ウィジェット作成:
  - `.autocomplete:async:pty:zle-widget` (zpty 内で使用)
  - `.autocomplete:async:pty:completion-widget` (zpty 内の補完)
  - `.autocomplete:async:complete:fd-widget` (FD callback)
  - `.autocomplete:async:wait:fd-widget` (遅延 callback)
  - `._list_choices` (結果表示用 completion widget)
  - `history-incremental-search-backward`, `recent-paths` (context toggle)
- ZLE フック登録:
  - `line-init` → `reset-context`
  - `line-pre-redraw` → `complete` (メイントリガー)
  - `line-finish` → `clear`
  - `isearch-update` → `isearch-update`
  - `isearch-exit` → `isearch-exit`

## 非同期アーキテクチャ

### データフロー

```
[ZLE フック: line-pre-redraw]
        │
        ▼
.autocomplete:async:complete
├── zle-flags チェック (yank/kill 検出)
├── KEYS_QUEUED_COUNT/PENDING チェック
├── REGION_ACTIVE / isearch チェック
├── LASTWIDGET の ignored リスト確認
└── .autocomplete:async:wait
        │
        ▼
[遅延: zselect -t (0.05s - overhead)]
        │  (FD ready 時)
        ▼
.autocomplete:async:wait:fd-widget
├── 状態一致確認 (same-state)
└── .autocomplete:async:start
        │
        ▼
[プロセス置換 <(...)]
        │
        ▼
.autocomplete:async:start:inner
├── フック無効化 (chpwd, precmd 等)
├── zpty AUTOCOMPLETE 起動
│   └── .autocomplete:async:pty
│       ├── TMOUT 設定 (timeout + 1s)
│       ├── bindkey ^@ → pty:zle-widget
│       └── vared __tmp__
│
├── zpty -w ^@ 送信
├── zpty -r header *^A 読み取り
│
├── [zpty 内で実行]:
│   └── .autocomplete:async:pty:zle-widget
│       ├── print ^A (ヘッダ開始)
│       ├── LBUFFER/RBUFFER 復元
│       ├── completion-widget 実行
│       └── print list_lines ^B (結果)
│
├── zselect -rt (timeout) で結果待ち
│   ├── 成功: zpty -r text *^B
│   └── タイムアウト: zpty -wn ^C^C^D (中断)
│
└── zpty -d AUTOCOMPLETE (クリーンアップ)
        │
        ▼ (print -rNC1)
.autocomplete:async:complete:fd-widget
├── 状態一致再確認
├── ._list_choices widget 実行
├── region_highlight 復元
├── autosuggest 再適用
└── zle -R (画面更新)
```

### 制御文字プロトコル

| 文字 | コード | 用途 |
|------|--------|------|
| `^@` | `\C-@` | zpty 内で ZLE widget をトリガー (bindkey 経由) |
| `^A` | `\C-A` | ヘッダ開始マーカー (widget 開始の合図) |
| `^B` | `\C-B` | 結果終了マーカー (データ末尾) |
| `^C` | `\C-C` | 中断 (タイムアウト時に2回送信) |
| `^D` | `\C-D` | 終了 (タイムアウト時に ^C^C の後) |

### FD 管理

```zsh
# 遅延用 FD (wait)
sysopen -r -o cloexec -u fd <(zselect -t $timeout; print)
zle -Fw "$fd" .autocomplete:async:wait:fd-widget

# 結果用 FD (complete)
sysopen -r -o cloexec -u _autocomplete__async_fd <(...)
zle -Fw "$_autocomplete__async_fd" .autocomplete:async:complete:fd-widget
```

- `sysopen -r -o cloexec` でファイルディスクリプタを開く
- `zle -Fw` で FD の読み取り可能時にウィジェットを呼び出す
- callback 内で即座に `zle -F $fd` でアンフック + `exec {fd}<&-` で close

### タイムアウト機構

2段階のタイムアウト:

1. **遅延タイムアウト** (wait):
   - デフォルト 0.05s
   - `_autocomplete__overhead` を差し引く
   - `zselect -t` で100分の1秒単位

2. **補完タイムアウト** (zpty):
   - デフォルト 1.0s
   - `zselect -rt` で結果待ち
   - タイムアウト時: `zpty -wn ^C^C^D` で強制終了
   - zpty 内の `TMOUT`: `timeout + 1` 秒で ALRM シグナル

### compadd シャドウイング

非同期補完では `compadd` を一時的に独自実装に差し替え:
```zsh
.autocomplete:async:shadow compadd   # 保存 + 差し替え
.autocomplete:async:shadow _describe

# ... 補完実行 ...

.autocomplete:async:unshadow compadd  # 復元
.autocomplete:async:unshadow _describe
```

独自 compadd の役割:
- `_autocomplete__max_lines` に基づき、表示可能な候補数を制限
- `_describe` 経由の場合: display 配列を cut して表示行数を制御
- 超過時: `comptags() { false }` で補完中断、`_autocomplete__partial_list` に記録

## Functions/Widgets/ 全5ファイル

### .autocomplete__complete-word__completion-widget (36行)

Tab 補完の中核 completion widget。

動作:
1. `curcontext` 設定 (history-incremental-search, recent-paths, 通常)
2. `comppostfuncs` に `.autocomplete__complete-word__post` を追加
3. 条件に応じて `compstate[old_list]` を制御
4. `autocomplete:_main_complete:new` を呼び出し

### .autocomplete__complete-word__post (69行)

補完後の処理 (comppostfunc)。

動作:
1. `MENUMODE` / `MENUSELECT` を unset
2. `compstate[list]` の制御 (menu → 'list force', otherwise → zle -Rc)
3. `compstate[to_end]` の制御
4. unambiguous 挿入の試行
5. menu/select/search モードの設定
6. `_autocomplete__should_add_space` による自動スペース追加
7. `_autocomplete__inserted` フラグ管理

### .autocomplete__down-line-or-select__zle-widget (7行)

```zsh
if [[ $RBUFFER == *$'\n'* ]]; then
    builtin zle down-line           # 複数行: 下の行に移動
else
    builtin zle menu-select -w      # 単一行: メニュー選択に移行
fi
```

### .autocomplete__history-search__completion-widget (22行)

履歴検索の completion widget。

動作:
1. `curcontext` を `history-search-backward:::` に設定
2. `_autocomplete__history_lines` を呼び出し
3. `comppostfuncs` で MENUSELECT=0, insert='menu:0' を設定

### .autocomplete__up-line-or-search__zle-widget (9行)

```zsh
if [[ $LBUFFER == *$'\n'* ]]; then
    builtin zle up-line              # 複数行: 上の行に移動
else
    builtin zle history-search-backward -w  # 単一行: 履歴検索
fi
```

## Functions/Util/ 全2ファイル

### .autocomplete__patch (8行)

関数をパッチする汎用ユーティリティ。

```zsh
# $1 の元の実装を autocomplete:${1}:old に保存
functions[autocomplete:${1}:old]="$(
    unfunction $1 2> /dev/null
    builtin autoload +X -Uz $1
    print -r -- "$functions[$1]"
)"
```

使用箇所: `_main_complete`, `_complete`, `_approximate` のパッチ。

### .autocomplete__zle-flags (28行)

ZLE フラグ管理。`line-pre-redraw` フック内で使用。

動作:
1. YANK_ACTIVE → `_autocomplete__zle_flags=yank[before]` → abort
2. CUTBUFFER 変更 → `_autocomplete__zle_flags=kill`
3. `zle -f` でフラグを ZLE に通知

## Completions/ 全7ファイル

### _autocomplete__command (6行)

`-command-` の補完。`_autocd` を呼び出すだけ。

### _autocomplete__compadd_opts_len (10行)

compadd のオプション部分の長さを計算するユーティリティ。`-` または `--` までのインデックスを返す。

### _autocomplete__history_lines (164行)

履歴検索の補完関数。

動作:
1. `fc -lrm` でパターンマッチする履歴を取得
2. `_comp_colors` でシンタックスハイライト設定
3. incremental 時: fuzzy sort (HISTNO 距離 + マッチ位置)、max 16 * list_lines バッファ
4. non-incremental 時: max = list_lines
5. `histfindnodups` オプション対応 (重複除去)
6. `compadd -QU -ld displays` で候補追加
7. menu-select 時: セミコロン自動付加 (`-S ';'`)

### _autocomplete__recent_paths (38行)

最近ディレクトリの補完関数。

動作:
1. `chpwd_recent_filehandler` で最近ファイル取得
2. `_autocomplete__max_lines` まで追加
3. ディレクトリと通常ファイルを分けて description 設定

### _autocomplete__should_add_space (15行)

補完後にスペースを自動付加するか判定。

判定基準: completags に `executables`, `aliases`, `functions`, `reserved-words`, `builtins`, `commands` のいずれかが含まれるか。

### _autocomplete__should_insert_unambiguous (9行)

unambiguous prefix を自動挿入するか判定。

条件:
- `_completer` が `_expand` でない
- widget が `*insert-unambiguous*` であるか、zstyle `insert-unambiguous` が true

### _autocomplete__unambiguous (43行)

unambiguous prefix を表示する補完関数。

動作:
1. history-lines / recent-* / partial_list / old_list keep / nmatches < 2 / expand → 早期リターン
2. `compstate[unambiguous]` が既にワードに含まれる → リターン
3. format でハイライト付きの "common substring: ..." を表示

## 全 Keybinding マップ

(詳細は [side-effects-comparison.md](./side-effects-comparison.md#3-keybinding-大量上書き) を参照)

合計: **main 6 + emacs 7 + vicmd 6 + menuselect 14 = 33 バインド**

## ZLE フック一覧

| フック | ウィジェット | 目的 |
|--------|-------------|------|
| `line-init` | `.autocomplete:async:reset-context` | コンテキストリセット、初回補完トリガー |
| `line-pre-redraw` | `.autocomplete:async:complete` | 非同期補完のメイントリガー |
| `line-finish` | `.autocomplete:async:clear` | コンテキストクリア、補完リスト消去 |
| `isearch-update` | `.autocomplete:async:isearch-update` | isearch 状態追跡 (フラグ ON) |
| `isearch-exit` | `.autocomplete:async:isearch-exit` | isearch 状態追跡 (フラグ OFF) |

## Completer チェーン

```
_expand → _complete → _complete:-fuzzy → _correct → _approximate → _ignored
```

### 各 Completer の設定

| Completer | 設定 |
|-----------|------|
| `_expand` | tag-order: `expansions all-expansions`, accept-exact: continue, glob: yes, keep-prefix: no |
| `_complete` | パッチ: `PREFIX=$PREFIX$SUFFIX; SUFFIX=` |
| `_complete:-fuzzy` | matcher-list: 3段階 (lower→upper+dot, +separator, +any) |
| `_correct` | tag-order: `! original` |
| `_approximate` | max-errors: `min(2, (PREFIX+SUFFIX)/3)`, compadd パッチ |
| `_ignored` | (標準動作) |

## zstyle 設定一覧

### .autocomplete__config で設定 (補完システム)

| zstyle パターン | キー | 値 |
|----------------|------|-----|
| `:completion:*` | `use-cache` | `yes` |
| `:completion:*` | `cache-path` | `$XDG_CACHE_HOME/zsh/compcache` (動的) |
| `:completion:*` | `completer` | `_expand _complete _complete:-fuzzy _correct _approximate _ignored` |
| `:completion:*` | `max-errors` | `min(2, (PREFIX+SUFFIX)/3)` (動的) |
| `:completion:*` | `matcher-list` | `m:{[:lower:]-}={[:upper:]_} r:\|[.]=**` |
| `:completion:*-fuzzy:*` | `matcher-list` | 3段階 (lower→upper+dot, +separator, +any) |
| `:completion:*:options` | `matcher` | `b:-=+` |
| `:completion:*` | `prefix-needed` | `yes` |
| `:completion:*:functions` | `ignored-patterns` | `*.*` `*:*` `+*` |
| `:completion:*:users` | `ignored-patterns` | `_*` |
| `:completion:*:widgets` | `ignored-patterns` | `*.*` `*:*` |
| `:completion:*` | `single-ignored` | `''` |
| `:completion:*:expand-alias:*` | `complete` | `yes` |
| `:completion:*:expand:*` | `tag-order` | `expansions all-expansions` |
| `:completion:*:expand:*` | `accept-exact` | `continue` |
| `:completion:*:expand:*` | `add-space` | `no` |
| `:completion:*:expand:*` | `glob` | `yes` |
| `:completion:*:expand:*` | `keep-prefix` | `no` |
| `:completion:*:expand:*` | `substitute` | `yes` |
| `:completion:*:expand:*` | `subst-globs-only` | `yes` |
| `:completion:*:-command-:*` | `tag-order` | (動的: path/command 別) |
| `:completion:*:-tilde-:*` | `tag-order` | `directory-stack named-directories` |
| `:completion:*:(approximate\|correct):*` | `tag-order` | `! original` |
| `:completion:*:cd:*` | `complete-options` | `yes` |
| `:completion:*:cd:*` | `tag-order` | `! directory-stack` |
| `:completion:*:fc:*` | `tag-order` | `options` |
| `:completion:*:git-*:*` | `tag-order` | (動的: nmatches > 0 時のみ) |
| `:completion:*` | `ignore-parents` | `parent pwd directory` |
| `:completion:*:paths` | `expand` | `suffix` |
| `:completion:*:paths` | `list-suffixes` | `yes` |
| `:completion:*:paths` | `special-dirs` | `no` |
| `:completion:*` | `group-name` | `''` |
| `:completion:*:-command-:*` | `group-name` | `commands` |
| `:completion:*:all-expansions` | `group-name` | `expansion` |
| `:completion:*` | `group-order` | `expansions options aliases ... directories executables` |
| `:completion:*` | `file-patterns` | `*(-/):directories %p(#q^-/):globbed-files` |
| `:completion:*:-command-:*` | `file-patterns` | (動的) |
| `:completion:*:(.\|source):*` | `file-patterns` | (*.zwc 除外) |
| `:completion:*:parameters` | `list-grouped` | `no` |
| `:completion:*:descriptions` | `format` | `%{\e[0;1;2m%}%d%{\e[0m%}` |
| `:completion:*:warnings` | `format` | `no matching %d completions` |
| `:completion:*:messages` | `format` | `%F{9}%d%f` |
| `:completion:*:history-lines` | `format` | `''` |
| `:completion:*` | `auto-description` | `%d` |
| `:completion:*:parameters` | `extra-verbose` | `yes` |
| `:completion:*:default` | `select-prompt` | `%F{black}%K{12}line %l %p%f%k` |
| `:completion:*` | `insert-sections` | `yes` |
| `:completion:*` | `separate-sections` | `yes` |
| `:completion:*` | `command` | `- COLUMNS=999` (Zsh < 5.9) |

### precmd で削除される設定
| zstyle パターン | キー | 処理 |
|----------------|------|------|
| `*` | `menu` | 全パターンから削除 |
| `*` | `list-prompt` | 全パターンから削除 |
| `:completion:*:*:*:*:default` | `menu` | `no no-select` に強制設定 |

### ランタイムで参照される設定 (ユーザーカスタマイズ可能)

| zstyle パターン | キー | デフォルト | 用途 |
|----------------|------|-----------|------|
| `:autocomplete:$mod` | `enabled` | `true` | モジュール有効/無効 |
| `:autocomplete:` | `default-context` | (なし) | デフォルト補完コンテキスト |
| `:autocomplete:` | `delay` / `min-delay` | `0.05` | 非同期補完の遅延 (秒) |
| `:autocomplete:$context` | `timeout` | `1.0` | 補完タイムアウト (秒) |
| `:autocomplete:$context:` | `min-input` | `1` (context あり: `0`) | 最小入力文字数 |
| `:autocomplete:$context:` | `ignored-input` | (なし) | 無視するパターン |
| `:autocomplete:$context:` | `list-lines` | `16` | 最大表示行数 |
| `:autocomplete:$curcontext` | `insert-unambiguous` | (なし) | unambiguous 自動挿入 |
| `:autocomplete:$WIDGET:` | `add-space` | `executables aliases functions builtins reserved-words commands` | 自動スペース付加対象 |
| `:autocomplete:$context:history-lines` | `add-semicolon` | `true` | セミコロン自動付加 |
| `:autocomplete::compinit` | `arguments` | (なし) | compinit 追加引数 |
| `:autocomplete:$LASTWIDGET:` | `ignore` | (なし) | 特定ウィジェット後の補完無効化 |
| `:chpwd:` | `recent-dirs-file` | `$XDG_DATA_HOME/zsh/chpwd-recent-dirs` | 最近ディレクトリファイル |
| `:chpwd:` | `recent-dirs-max` | `0` (無制限) | 最大保存数 |

## グローバル変数

| 変数 | 型 | 用途 |
|------|-----|------|
| `_autocomplete__func_opts` | array | 共通 setopt オプション |
| `_autocomplete__ctxt_opts` | array | コンテキスト用 setopt (completealiases, completeinword) |
| `_autocomplete__mods` | array | モジュール名リスト |
| `_autocomplete__funcfiletrace` | array | 元の funcfiletrace |
| `_autocomplete__log` | string | ログファイルパス |
| `_autocomplete__ps4` | string | デバッグ用 PS4 |
| `_autocomplete__compdef` | array | 遅延 compdef 呼び出し |
| `_autocomplete__suggest_ignore_widgets` | array | Autosuggest 無視リスト |
| `_autocomplete__overhead` | number | 前回の補完オーバーヘッド |
| `_autocomplete__region_highlight` | array | 保存された region_highlight |
| `_autocomplete__last_cutbuffer` | string | CUTBUFFER 変更検知用 |
| `_autocomplete__zle_flags` | string | ZLE フラグ (yank/kill) |
| `_autocomplete__curcontext` | string | 保存されたコンテキスト |
| `_autocomplete__lbuffer` | string | 保存された LBUFFER (状態比較用) |
| `_autocomplete__rbuffer` | string | 保存された RBUFFER |
| `_autocomplete__async_fd` | number | 非同期結果の FD |
| `_autocomplete__isearch` | number | isearch 状態フラグ |
| `_autocomplete__inserted` | - | 挿入フラグ (set/unset) |
| `_autocomplete__partial_list` | string | 部分リストの curtag |
| `_autocomplete__max_lines` | number | 最大表示行数 |
| `_autocomplete__reserved_lines` | number | 予約行数 |
| `ZLE_REMOVE_SUFFIX_CHARS` | string | 接尾辞削除文字 |
| `ZLE_SPACE_SUFFIX_CHARS` | string | スペース接尾辞文字 |
| `ZSH_AUTOSUGGEST_USE_ASYNC` | string | Autosuggest 非同期モード |
| `ZSH_AUTOSUGGEST_MANUAL_REBIND` | string | Autosuggest 手動 rebind |
| `ZSH_AUTOSUGGEST_ORIGINAL_WIDGET_PREFIX` | string | Autosuggest prefix |

## カスタムウィジェット一覧

### ZLE ウィジェット (7個)

| ウィジェット | 実装 |
|-------------|------|
| `up-line-or-search` | `.autocomplete__up-line-or-search__zle-widget` |
| `down-line-or-select` | `.autocomplete__down-line-or-select__zle-widget` |
| `history-search-backward` | (zle-widget, 未実装 → history-search) |
| `.autocomplete:async:pty:zle-widget` | zpty 内の ZLE 制御 |
| `.autocomplete:async:complete:fd-widget` | 非同期結果受信 |
| `.autocomplete:async:wait:fd-widget` | 遅延 callback |
| `history-incremental-search-backward` / `recent-paths` | context toggle |

### Completion ウィジェット (9個)

| ウィジェット | 種別 | 実装 |
|-------------|------|------|
| `complete-word` | complete-word | `.autocomplete__complete-word__completion-widget` |
| `menu-complete` | menu-complete | 同上 |
| `menu-select` | menu-select | 同上 |
| `reverse-menu-complete` | complete-word | 同上 |
| `insert-unambiguous-or-complete` | complete-word | 同上 |
| `menu-search` | menu-select | 同上 |
| `history-search-backward` | menu-select | `.autocomplete__history-search__completion-widget` |
| `.autocomplete:async:pty:completion-widget` | list-choices | zpty 内補完 |
| `._list_choices` | list-choices | 非同期結果表示 |
