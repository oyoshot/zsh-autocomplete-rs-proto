# 副作用・問題点の詳細分析

## zsh-autocomplete の副作用一覧

### 1. グローバルオプション変更

| オプション        | 変更箇所                               | 影響                                   |
| ----------------- | -------------------------------------- | -------------------------------------- |
| `autopushd`       | `.autocomplete__recent-dirs:precmd`    | `cd` でディレクトリスタックに自動 push |
| `pushdignoredups` | `.autocomplete__recent-dirs:precmd`    | スタック内の重複を無視                 |
| `NO_listbeep`     | `plugin.zsh` (L1: `unsetopt listbeep`) | 補完リスト表示時のビープ音を無効化     |

### 2. 環境変数変更

| 変数                                     | 値                                   | 変更箇所                 |
| ---------------------------------------- | ------------------------------------ | ------------------------ |
| `ZLE_REMOVE_SUFFIX_CHARS`                | `$' /;\n\r\t'`                       | `.autocomplete__config`  |
| `ZLE_SPACE_SUFFIX_CHARS`                 | `'\|&<>-+'`                          | `.autocomplete__config`  |
| `FPATH` / `fpath`                        | Completions ディレクトリを先頭に追加 | `.autocomplete__main`    |
| `ZSH_AUTOSUGGEST_USE_ASYNC`              | `yes`                                | `.autocomplete__async`   |
| `ZSH_AUTOSUGGEST_MANUAL_REBIND`          | `1`                                  | `.autocomplete__widgets` |
| `ZSH_AUTOSUGGEST_ORIGINAL_WIDGET_PREFIX` | `.autosuggest-orig-`                 | `.autocomplete__widgets` |

### 3. Keybinding 大量上書き

#### main keymap

| キー                    | ウィジェット              | 用途           |
| ----------------------- | ------------------------- | -------------- |
| `\t` (Tab)              | `complete-word`           | Tab 補完       |
| `Shift+Tab`             | `expand-word`             | 展開           |
| `Up` (`\e[A`, `\eOA`)   | `up-line-or-search`       | 上移動/検索    |
| `Down` (`\e[B`, `\eOB`) | `down-line-or-select`     | 下移動/選択    |
| `Alt+Up`                | `history-search-backward` | 履歴逆方向検索 |
| `Alt+Down`              | `menu-select`             | メニュー選択   |

#### emacs keymap

| キー  | ウィジェット                          |
| ----- | ------------------------------------- |
| `^P`  | `up-line-or-search`                   |
| `^N`  | `down-line-or-select`                 |
| `\ep` | `history-search-backward`             |
| `\en` | `menu-select`                         |
| `^R`  | `history-incremental-search-backward` |
| `^S`  | `menu-search`                         |
| `^X/` | `recent-paths`                        |

#### vicmd keymap

| キー | ウィジェット                          |
| ---- | ------------------------------------- |
| `k`  | `up-line-or-search`                   |
| `j`  | `down-line-or-select`                 |
| `^P` | `history-search-backward`             |
| `^N` | `menu-select`                         |
| `/`  | `history-incremental-search-backward` |
| `?`  | `menu-search`                         |

#### menuselect keymap

| キー              | ウィジェット                          |
| ----------------- | ------------------------------------- |
| `\t` (Tab)        | `menu-complete`                       |
| `Shift+Tab`       | `reverse-menu-complete`               |
| `^@` (Ctrl+Space) | `accept-and-hold`                     |
| `\ev`             | `accept-and-hold`                     |
| `^_`              | `undo`                                |
| `\eu`             | `undo`                                |
| `PageUp`          | `backward-word`                       |
| `PageDown`        | `forward-word`                        |
| `Up`              | `up-history`                          |
| `Down`            | `down-history`                        |
| `^P` / `Alt+Up`   | `vi-backward-blank-word`              |
| `^N` / `Alt+Down` | `vi-forward-blank-word`               |
| `^R`              | `history-incremental-search-backward` |
| `^S`              | `history-incremental-search-forward`  |

### 4. compinit 多重実行・パッチ

- `compinit` を precmd 内で条件付き再実行
- `compinit()` を空関数で上書きし、他プラグインによる再実行を抑止
- `_main_complete` をパッチ:
  - `compstate[insert]=automenu-unambiguous`
  - `compstate[last_prompt]=yes`
  - `compstate[list]='list force packed'`
  - `TRAPINT` / `TRAPQUIT` を再定義
  - `compprefuncs` / `comppostfuncs` にフックを挿入
- `_complete` をパッチ: `PREFIX=$PREFIX$SUFFIX; SUFFIX=` で修正
- `_approximate` をパッチ: compadd 関数の一時差し替え
- `bindkey` を一時的に空関数にして compinit 内の keybinding 登録を抑制

### 5. Completer チェーン拡張

```
_expand → _complete → _complete:-fuzzy → _correct → _approximate → _ignored
```

- `_complete:-fuzzy` を追加（通常の zsh にはない）
- fuzzy 用に独自の `matcher-list` を設定
- max-errors を動的計算: `min(2, (PREFIX + SUFFIX) / 3)`

### 6. CDPATH クリア

```zsh
# .autocomplete__compinit:precmd
[[ -v CDPATH && -z $CDPATH ]] && unset CDPATH cdpath
```

空の CDPATH を unset する。

### 7. TRAPINT/TRAPQUIT 上書き

```zsh
# autocomplete:_main_complete:new:pre
TRAPINT() { zle -M "${(F)funcfiletrace}"; zle -R; return 130 }
TRAPQUIT() { zle -M "${(F)funcfiletrace}"; zle -R; return 131 }
```

補完中のシグナルハンドラを独自実装に置換。

### 8. precmd フック登録

- `add-zsh-hook precmd .autocomplete__main:precmd`
- precmd 内で各モジュールの初期化を実行
- `precmd_functions` の先頭に挿入（autosuggest より前に実行するため）

### 9. ZLE フック登録

| フック            | ウィジェット                         |
| ----------------- | ------------------------------------ |
| `line-init`       | `.autocomplete:async:reset-context`  |
| `line-pre-redraw` | `.autocomplete:async:complete`       |
| `line-finish`     | `.autocomplete:async:clear`          |
| `isearch-update`  | `.autocomplete:async:isearch-update` |
| `isearch-exit`    | `.autocomplete:async:isearch-exit`   |

### 10. ファイルシステム副作用

| パス                                    | 内容                                     |
| --------------------------------------- | ---------------------------------------- |
| `$XDG_STATE_HOME/zsh-autocomplete/log/` | 日付別ログファイル (7日でローテーション) |
| `$XDG_CACHE_HOME/zsh/compcache/`        | 補完キャッシュ + zcompile                |
| `$XDG_CACHE_HOME/zsh/compdump`          | compinit ダンプファイル                  |
| `$XDG_DATA_HOME/zsh/chpwd-recent-dirs`  | 最近ディレクトリ                         |

### 11. zstyle 大量設定

`.autocomplete__config` で 70+ の zstyle 設定を行う：

- `use-cache`, `cache-path`, `completer`, `max-errors`, `matcher-list`
- `prefix-needed`, `ignored-patterns`, `single-ignored`
- `tag-order` (command, tilde, approximate, cd, fc, git)
- `ignore-parents`, `group-name`, `group-order`
- `file-patterns`, `format`, `auto-description`, `select-prompt`
- `insert-sections`, `separate-sections`
- precmd で `menu`, `list-prompt` を強制削除

### 12. カスタムウィジェット登録

**ZLE ウィジェット (7個)**:

- `up-line-or-search`
- `down-line-or-select`
- `history-search-backward`
- `.autocomplete:async:pty:zle-widget`
- `.autocomplete:async:complete:fd-widget`
- `.autocomplete:async:wait:fd-widget`
- `history-incremental-search-backward`, `recent-paths` (toggle-context)

**Completion ウィジェット (9個)**:

- `complete-word`, `menu-complete`, `menu-select` (同一実装)
- `reverse-menu-complete`
- `insert-unambiguous-or-complete`
- `menu-search`
- `history-search-backward` (completion widget)
- `.autocomplete:async:pty:completion-widget`
- `._list_choices`

---

## shell-popup-sandbox の副作用一覧

### 1. 全 ASCII printable rebind

```zsh
for i in {32..126}; do
    bindkey "$char" shell-popup-self-insert
done
```

**95個のキー** (スペースからチルダまで) を `shell-popup-self-insert` に rebind。これは非常に侵襲的で、他のプラグインや設定と衝突する。

### 2. Tab/Backspace/Arrow 上書き

| キー             | ウィジェット                       |
| ---------------- | ---------------------------------- |
| `^I` (Tab)       | `shell-popup-tab-complete`         |
| `^?` (Backspace) | `shell-popup-backward-delete-char` |
| `\e[D` (Left)    | `shell-popup-backward-char`        |
| `\e[C` (Right)   | `shell-popup-forward-char`         |

### 3. 候補収集の制限

- glob (`${prefix}*(N)`) とコマンドハッシュ (`${(k)commands}`) のみ
- zsh 補完システムを使わないため、サブコマンドやオプション補完ができない
- description なし

---

## zsh-autocomplete-rs での抑制戦略

| 元の副作用                                            | 本プロジェクトの対策                                                       |
| ----------------------------------------------------- | -------------------------------------------------------------------------- |
| **グローバルオプション変更** (autopushd, listbeep 等) | 一切変更しない                                                             |
| **ZLE_REMOVE_SUFFIX_CHARS 等の変更**                  | 一切変更しない                                                             |
| **FPATH 変更**                                        | 一切変更しない                                                             |
| **keybinding 大量上書き** (15+ keys)                  | `^I` (Tab) のみ上書き。自動トリガーは `line-pre-redraw` ZLE フック         |
| **全 ASCII rebind** (sandbox の問題)                  | `line-pre-redraw` フックに置換。キーバインドへの介入なし                   |
| **compinit 多重実行**                                 | compinit を呼ばない。ユーザー環境の補完システムをそのまま利用              |
| **compinit パッチ**                                   | `_main_complete` 等にパッチしない                                          |
| **completer チェーン拡張**                            | ユーザーの completer 設定をそのまま使用                                    |
| **CDPATH クリア**                                     | CDPATH に触れない                                                          |
| **TRAPINT/TRAPQUIT 上書き**                           | trap 不使用。Ctrl+C は Rust 側の raw mode で処理                           |
| **precmd フック**                                     | 不使用。初期化は `source` 時に完結                                         |
| **ZLE フック大量登録** (5個)                          | `line-pre-redraw` のみ 1 個                                                |
| **ファイルシステム副作用**                            | ログ・キャッシュ・ダンプファイルなし (Phase 4 で設定ファイルのみ)          |
| **zstyle 大量設定** (70+)                             | zstyle に触れない                                                          |
| **カスタムウィジェット大量登録** (16個)               | 最小限 (`_zacrs_tab_complete`, `_zacrs_line_pre_redraw`)                   |
| **候補収集の制限** (sandbox)                          | glob + commands/aliases/builtins/functions。将来は zpty + compadd override |
| **Autosuggest との統合コード**                        | 不要。副作用がないため衝突しない                                           |

### 設計原則

1. **ユーザー環境を尊重**: グローバル状態を変更しない
2. **最小限の footprint**: keybinding は `^I` のみ、ZLE フックは `line-pre-redraw` のみ
3. **隔離された実行**: Rust バイナリは `/dev/tty` に直接描画し、シェルの IO と干渉しない
4. **安全な終了**: TtyGuard による raw mode 確実復元、panic 時も安全
5. **フォールバック**: 候補がない場合は元の `expand-or-complete` にフォールバック
