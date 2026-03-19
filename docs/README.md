# zsh-autocomplete-rs ドキュメント

## プロジェクト概要

**zsh-autocomplete-rs** は、[zsh-autocomplete](https://github.com/marlonrichert/zsh-autocomplete) の Rust 製代替プラグイン。

### なぜ作るのか

zsh-autocomplete は高機能だが、以下の問題がある：

- **副作用が多い**: keybinding の大量上書き、compinit 多重実行、グローバルオプション変更など
- **他プラグインとの衝突**: zsh-autosuggestions や fast-syntax-highlighting と干渉
- **デバッグが困難**: 複雑な非同期アーキテクチャ、パッチだらけの completer チェーン

### 何を作るのか

- Rust バイナリ + Zsh プラグインのハイブリッド構成
- テキストエディタ風のポップアップ UI（VSCode のような候補表示）
- 副作用を最小限に抑制（`^I` のみ上書き、`line-pre-redraw` フック使用）
- nucleo による高速ファジーマッチング

## ドキュメント一覧

| ファイル | 概要 |
|----------|------|
| [zsh-autocomplete-analysis.md](./zsh-autocomplete-analysis.md) | 元プラグインの完全分析（初期化フロー、全ファイル、keybinding、非同期アーキテクチャ等） |
| [shell-popup-sandbox-analysis.md](./shell-popup-sandbox-analysis.md) | プロトタイプの完全分析（Rust 5ファイル + Zsh スクリプト、定数、エッジケース） |
| [side-effects-comparison.md](./side-effects-comparison.md) | 副作用・問題点の詳細比較と抑制戦略 |
| [implementation-guide.md](./implementation-guide.md) | 代替の実現方法（アーキテクチャ、通信フロー、統合詳細） |
| [development-roadmap.md](./development-roadmap.md) | 開発ロードマップ（フェーズ計画、ファイル構成、テスト戦略） |

## ディレクトリ構成

```
docs/
  README.md                          # このファイル（ドキュメント索引）
  zsh-autocomplete-analysis.md       # 元プラグイン分析
  shell-popup-sandbox-analysis.md    # プロトタイプ分析
  side-effects-comparison.md         # 副作用比較
  implementation-guide.md            # 実現方法
  development-roadmap.md             # 開発ロードマップ

src/                                 # Rust ソース (Phase 1)
shell/                               # Zsh プラグイン (Phase 1)
tmp/
  zsh-autocomplete/                  # 元プラグイン (リファレンス)
  shell-popup-sandbox/               # プロトタイプ (リファレンス)
```
