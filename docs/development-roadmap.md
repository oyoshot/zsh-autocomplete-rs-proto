# 開発ロードマップ

## フェーズ計画

### Phase 1: Tab補完 + 自動トリガー + ポップアップUI（現在）

**ゴール**: `source plugin.zsh` して Tab でポップアップが出る

#### 必要ファイル

**Rust 側 (src/)**:
| ファイル | 役割 |
|----------|------|
| `main.rs` | clap サブコマンド解析、stdin 読み込み、イベントループ、終了コード |
| `cli.rs` | clap 引数定義 (`complete --prefix --cursor-row --cursor-col`) |
| `tty.rs` | `/dev/tty` 管理、TtyGuard (raw mode 確実復元) |
| `app.rs` | App 状態 (候補、フィルタ、スクロール、選択) |
| `candidate.rs` | `Candidate { text, description }`、stdin 解析 (tab区切り) |
| `fuzzy.rs` | nucleo FuzzyMatcher、Candidate 対応 |
| `input.rs` | crossterm キーイベント → Action 変換 |
| `config.rs` | 設定構造体 (将来拡張用) |
| `ui/mod.rs` | UI モジュール公開 |
| `ui/popup.rs` | Popup 位置計算 (上下自動判定、画面内収め) |
| `ui/render.rs` | draw/clear (ボックス描画、description 列、選択ハイライト) |
| `ui/theme.rs` | 色・スタイル定義 |

**Zsh 側 (shell/)**:
| ファイル | 役割 |
|----------|------|
| `zsh-autocomplete-rs.plugin.zsh` | メインプラグイン (widgets, keybind, hooks) |
| `_zacrs_gather.zsh` | 候補収集 (glob + commands/aliases/builtins/functions) |
| `_zacrs_util.zsh` | ヘルパー (CPR カーソル位置取得, prefix 抽出) |

#### 依存クレート
- `crossterm` 0.28 - ターミナル操作 (raw mode, カーソル制御, 描画)
- `nucleo-matcher` 0.3 - ファジーマッチング
- `unicode-width` 0.2 - Unicode 文字幅計算
- `clap` 4 (derive) - CLI 引数解析

#### マイルストーン
1. `cargo build` でバイナリが生成される
2. `echo "foo\nbar\nbaz" | zsh-autocomplete-rs complete --prefix f --cursor-row 5 --cursor-col 10` でポップアップが出る
3. `source shell/zsh-autocomplete-rs.plugin.zsh` して Tab で補完が動く
4. `line-pre-redraw` で自動トリガーが動く

### Phase 2: 履歴検索・最近ディレクトリ

**ゴール**: Ctrl+R で履歴検索、Ctrl+X / で最近ディレクトリ

- zsh-autocomplete の `_autocomplete__history_lines` 相当
  - `fc -lrm` でパターン一致する履歴を取得
  - ファジーソート、重複除去
- zsh-autocomplete の `_autocomplete__recent_paths` 相当
  - `chpwd_recent_dirs` / `chpwd_recent_filehandler` で最近ディレクトリを取得
- Rust サブコマンド追加: `history-search`, `recent-dirs`
- Zsh 側: 新しい ZLE widget、keybinding 追加

### Phase 3: 非同期補完・スマート挿入

**ゴール**: バックグラウンド補完、unambiguous prefix 自動挿入

- `line-pre-redraw` フックからバックグラウンドで候補収集
  - zpty 内で補完を実行、完了後にポップアップ表示
  - 遅延 (delay) 設定で入力中のちらつき防止
- スマート挿入
  - unambiguous prefix の自動挿入
  - space/semicolon 自動付加 (executables, commands 等)
- zsh-autocomplete の `_autocomplete__unambiguous`, `_autocomplete__should_add_space` 相当

### Phase 4: パフォーマンス・設定・拡張

**ゴール**: 実用レベルのポリッシュ

- キャッシュ
  - 候補リストのキャッシュ (同一 prefix に対する重複計算を回避)
  - TTL ベースの無効化
- パフォーマンス最適化
  - 大量候補のストリーミング処理
  - 描画の差分更新
- TOML 設定システム
  - `~/.config/zsh-autocomplete-rs/config.toml`
  - テーマ、max_visible、delay、keybinding カスタマイズ
- vi-mode 対応
  - vicmd keymap での keybinding
  - insert/normal モード切り替え追跡

## 移植元マッピング表

### sandbox → 本プロジェクト

| 移植先 | 移植元 (tmp/shell-popup-sandbox/) | 変更点 |
|--------|----------------------------------|--------|
| `src/main.rs` | `src/main.rs` | 手動引数解析 → clap、`--candidates` → stdin、TtyGuard 化 |
| `src/cli.rs` | (新規) | clap derive による引数定義 |
| `src/tty.rs` | `src/main.rs` (tty部分) | TtyGuard に分離、Drop で確実復元 |
| `src/app.rs` | `src/app.rs` | `Vec<String>` → `Vec<Candidate>` (text + description) |
| `src/candidate.rs` | (新規) | stdin tab区切り解析、Candidate 構造体 |
| `src/fuzzy.rs` | `src/fuzzy.rs` | `String` → `Candidate` 対応 |
| `src/input.rs` | `src/input.rs` | ほぼそのまま |
| `src/config.rs` | (新規) | 設定構造体 (将来拡張用) |
| `src/ui/popup.rs` | `src/ui.rs` (Popup部分) | 分離のみ |
| `src/ui/render.rs` | `src/ui.rs` (draw/clear) | description 列追加、gap 埋め |
| `src/ui/theme.rs` | (新規) | 色定義を分離 |
| `shell/*.zsh` | `shell-popup.zsh` | 分割 (plugin + gather + util)、zpty候補収集、line-pre-redraw 方式 |

### zsh-autocomplete → 本プロジェクト（将来フェーズ）

| 本プロジェクトの機能 | zsh-autocomplete の対応ファイル | Phase |
|---------------------|-------------------------------|-------|
| Tab 補完 | `.autocomplete__widgets`, `complete-word` | 1 |
| 自動トリガー | `.autocomplete__async` (line-pre-redraw) | 1 |
| 候補収集 | `.autocomplete__compinit`, `_main_complete` | 1 (簡易版) |
| 履歴検索 | `_autocomplete__history_lines` | 2 |
| 最近ディレクトリ | `_autocomplete__recent_paths`, `.autocomplete__recent-dirs` | 2 |
| 非同期補完 | `.autocomplete__async` (zpty + FD callback) | 3 |
| unambiguous 挿入 | `_autocomplete__unambiguous`, `_autocomplete__should_insert_unambiguous` | 3 |
| space 自動付加 | `_autocomplete__should_add_space` | 3 |

## ファイル構成計画

```
zsh-autocomplete-rs/
├── Cargo.toml
├── src/
│   ├── main.rs           # エントリポイント、サブコマンド分岐
│   ├── cli.rs            # clap 引数定義
│   ├── tty.rs            # /dev/tty、TtyGuard
│   ├── app.rs            # App 状態管理
│   ├── candidate.rs      # Candidate 構造体、stdin 解析
│   ├── fuzzy.rs          # FuzzyMatcher (nucleo)
│   ├── input.rs          # キー入力 → Action
│   ├── config.rs         # 設定 (将来用)
│   └── ui/
│       ├── mod.rs        # モジュール公開
│       ├── popup.rs      # Popup 位置計算
│       ├── render.rs     # 描画 (draw/clear)
│       └── theme.rs      # 色・スタイル
├── shell/
│   ├── zsh-autocomplete-rs.plugin.zsh  # メインプラグイン
│   ├── _zacrs_gather.zsh              # 候補収集
│   └── _zacrs_util.zsh               # ヘルパー
├── docs/                              # ドキュメント
└── tmp/                               # リファレンス (git 管理外)
    ├── zsh-autocomplete/              # 元プラグイン
    └── shell-popup-sandbox/           # プロトタイプ
```

## テスト戦略

### Rust 単体テスト
- `fuzzy.rs`: 空クエリ、マッチ、ノーマッチ、スコア順
- `candidate.rs`: tab区切り解析、description なし、空行
- `app.rs`: スクロール (循環、自動調整)、フィルタ更新
- `ui/popup.rs`: 位置計算 (上下判定、画面端)

### 統合テスト (Zsh)
- `shell/` スクリプトのテスト
  - `_zacrs_gather.zsh`: 候補収集が期待通りか
  - `_zacrs_util.zsh`: CPR パース、prefix 抽出
  - `plugin.zsh`: source 後に widget が登録されるか、keybinding が正しいか
- 手動テスト
  - Tab でポップアップが出る
  - 自動トリガーが動く
  - 結果が LBUFFER に反映される
