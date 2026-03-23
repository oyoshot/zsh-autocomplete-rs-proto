window.BENCHMARK_DATA = {
  "lastUpdate": 1774245366131,
  "repoUrl": "https://github.com/oyoshot/zsh-autocomplete-rs-proto",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2310f0ef3f95a290e4e7c1a5935b0cdbc7aa3360",
          "message": "Merge pull request #1 from oyoshot/ci/setup-github-actions\n\nci: add GitHub Actions workflows and dependabot config",
          "timestamp": "2026-03-20T21:38:26+09:00",
          "tree_id": "bf4b235e2fca6de4d6844ee453b5019cf7fa17ba",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/2310f0ef3f95a290e4e7c1a5935b0cdbc7aa3360"
        },
        "date": 1774010764144,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 9395,
            "range": "± 1220",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 92633,
            "range": "± 2360",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 1121465,
            "range": "± 63691",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 222504,
            "range": "± 2351",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 124156,
            "range": "± 3030",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 96480,
            "range": "± 5081",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 25240,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 39215,
            "range": "± 722",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 196631,
            "range": "± 869",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 21637,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 322,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 159,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 319,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2398,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 227,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 96,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 98,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 101,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 57,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 138,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 863,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7550,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 773,
            "range": "± 6",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "36abc8447a805e5481fba8326eac875f3c08e116",
          "message": "feat(theme): add [theme] config for popup color customization (#5)\n\n* docs(roadmap): update development phases and clarify scope\n\n- mark Phase 1 as completed\n- move smart insertion and UX work into Phase 2\n- defer async completion to Phase 3 and polish to Phase 4\n- declare history search and recent directory features out of scope\n\n* feat(theme): add [theme] config section for popup color customization\n\nAdd ANSI color defaults with config.toml override support for 6 styling\npoints: border, selected-fg/bg, description, filter, and candidate.\nDefaults preserve current appearance; invalid values gracefully fallback.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(ui): extract render_popup helper and reduce theme boilerplate\n\n- Extract shared popup rendering from draw()/draw_popup_only() into\n  render_popup(), eliminating ~170 lines of duplicated code.\n- Add print_colored() helper for the repeated color-or-plain pattern.\n- Make ThemeRaw and Config.theme_raw private (implementation detail).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(config): tighten visibility of ThemeRaw fields and parse_color\n\n- Remove pub from ThemeRaw fields (struct is already private)\n- Narrow parse_color from pub to pub(crate)\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-20T23:33:53+09:00",
          "tree_id": "c801671eb1d88172f4564c183c5384495b240ab1",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/36abc8447a805e5481fba8326eac875f3c08e116"
        },
        "date": 1774017698857,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 9528,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 90587,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 1109433,
            "range": "± 137754",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 223237,
            "range": "± 3311",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 115180,
            "range": "± 1300",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 94822,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 25119,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 39954,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 188226,
            "range": "± 2704",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 21247,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 315,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 159,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 317,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2365,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 225,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 92,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 96,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 33,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 77,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 138,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 863,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7554,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 774,
            "range": "± 25",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "877dd7ea3fca65f971456f35401f2930f8adb062",
          "message": "chore(deps): bump crossterm from 0.28.1 to 0.29.0 (#3)\n\nBumps [crossterm](https://github.com/crossterm-rs/crossterm) from 0.28.1 to 0.29.0.\n- [Release notes](https://github.com/crossterm-rs/crossterm/releases)\n- [Changelog](https://github.com/crossterm-rs/crossterm/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/crossterm-rs/crossterm/commits/0.29)\n\n---\nupdated-dependencies:\n- dependency-name: crossterm\n  dependency-version: 0.29.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-20T23:43:55+09:00",
          "tree_id": "f27038e14d86a9d7157b840183b8271147682c3c",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/877dd7ea3fca65f971456f35401f2930f8adb062"
        },
        "date": 1774018297488,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7880,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 91813,
            "range": "± 860",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 1126244,
            "range": "± 52136",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 247270,
            "range": "± 1617",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 130303,
            "range": "± 7008",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 102598,
            "range": "± 814",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 26380,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 42820,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 191161,
            "range": "± 8374",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 22642,
            "range": "± 1822",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 344,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 211,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 337,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2406,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 261,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 95,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 99,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 100,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 34,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 74,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 136,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 849,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7498,
            "range": "± 515",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 736,
            "range": "± 6",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5f31cf05567d07449c7781413bd2edb40588012d",
          "message": "chore(deps): bump toml from 0.8.23 to 1.0.7+spec-1.1.0 (#2)\n\nBumps [toml](https://github.com/toml-rs/toml) from 0.8.23 to 1.0.7+spec-1.1.0.\n- [Commits](https://github.com/toml-rs/toml/compare/toml-v0.8.23...toml-v1.0.7)\n\n---\nupdated-dependencies:\n- dependency-name: toml\n  dependency-version: 1.0.7+spec-1.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-major\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-20T23:52:38+09:00",
          "tree_id": "64afb111462437c2fb116ab22bfd6d971b3ceff0",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/5f31cf05567d07449c7781413bd2edb40588012d"
        },
        "date": 1774018820226,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 9646,
            "range": "± 1274",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 91929,
            "range": "± 6869",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 1102791,
            "range": "± 55199",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 222670,
            "range": "± 2266",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 117021,
            "range": "± 652",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 81828,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 25138,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 40457,
            "range": "± 1894",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 189992,
            "range": "± 9267",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 21744,
            "range": "± 212",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 314,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 158,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 316,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2414,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 228,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 98,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 98,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 93,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 55,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 73,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 140,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 861,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7569,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 773,
            "range": "± 9",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9cd5c14887e14ff2d981c4bd6a6a81c161de2a14",
          "message": "chore(deps): bump criterion from 0.5.1 to 0.8.2 (#4)\n\nBumps [criterion](https://github.com/criterion-rs/criterion.rs) from 0.5.1 to 0.8.2.\n- [Release notes](https://github.com/criterion-rs/criterion.rs/releases)\n- [Changelog](https://github.com/criterion-rs/criterion.rs/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/criterion-rs/criterion.rs/compare/0.5.1...criterion-v0.8.2)\n\n---\nupdated-dependencies:\n- dependency-name: criterion\n  dependency-version: 0.8.2\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-21T01:13:02+09:00",
          "tree_id": "ca08474a1318af581054d9e5625b7fee4bde4fa8",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/9cd5c14887e14ff2d981c4bd6a6a81c161de2a14"
        },
        "date": 1774023647811,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 9609,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 91864,
            "range": "± 2770",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 1124098,
            "range": "± 44601",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 224449,
            "range": "± 2060",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 127015,
            "range": "± 2099",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 97563,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 26494,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 41810,
            "range": "± 238",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 194272,
            "range": "± 6529",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 23154,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 313,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 164,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 316,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2393,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 233,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 102,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 32,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 142,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 854,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7525,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 735,
            "range": "± 7",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5a693e7a5c2d3fdfaf3c3d7b852a84914e15a542",
          "message": "ci(actions): run tests on Ubuntu and macOS (#6)\n\n- add an OS matrix for the test job with fail-fast disabled\n- use OS-specific rust-cache keys for matrix runs",
          "timestamp": "2026-03-21T13:55:12+09:00",
          "tree_id": "0f5ce2cb98fda3fdfef8ef71115cdf8fec670485",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/5a693e7a5c2d3fdfaf3c3d7b852a84914e15a542"
        },
        "date": 1774069370983,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 9610,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 93141,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 1114110,
            "range": "± 66726",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 224444,
            "range": "± 3262",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 107026,
            "range": "± 4281",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 97098,
            "range": "± 3217",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 26438,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 42442,
            "range": "± 1585",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 193047,
            "range": "± 2176",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 23066,
            "range": "± 1082",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 313,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 163,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 319,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2402,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 225,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 97,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 92,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 32,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 78,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 74,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 138,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 852,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7518,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 736,
            "range": "± 7",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oyoshot@gmail.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "oyoshot@gmail.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "distinct": true,
          "id": "a5f3a87068e39ceea26241235309ad23c97defc5",
          "message": "docs(agents): refresh repository guidelines for daemon and CLI flows",
          "timestamp": "2026-03-21T18:54:00+09:00",
          "tree_id": "c2bc1649118c842d7a3c8b211967f15b5e5cbfdb",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/a5f3a87068e39ceea26241235309ad23c97defc5"
        },
        "date": 1774087344852,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 8012,
            "range": "± 789",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 73221,
            "range": "± 703",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 943052,
            "range": "± 40698",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 194592,
            "range": "± 17585",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 106226,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 77248,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 16019,
            "range": "± 447",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 27307,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 160243,
            "range": "± 7182",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11320,
            "range": "± 168",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 278,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 140,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 280,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2159,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 199,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 33,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 101,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 39,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 98,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 136,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 859,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7570,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 761,
            "range": "± 13",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bf39c8178e7876bd2b07164c0119e13781a3cb5e",
          "message": "feat(shell): chain subcommand popup after completion confirmation (#9)\n\n* feat(shell): chain subcommand popup after completion confirmation\n\nWhen a candidate is confirmed (Enter) or a single candidate is auto-inserted,\nautomatically trigger the next subcommand popup if the result ends with a\nspace or slash. This chains completions so that selecting \"go\" immediately\nshows go's subcommands.\n\nFor lazy-loaded completions (uv, mise, op, etc.), retry compsys once when\nthe first call returns 0 candidates — the first call triggers the deferred\nload and the second retrieves results. This retry also fires at subcommand\nposition (empty naive_prefix) during normal typing, not just chains.\n\nAlso fix single-candidate auto-insert to add trailing space for\ncommand/alias/builtin/function/file kinds, matching Rust text_with_suffix().\nGuard gather fallback to require non-empty naive_prefix, preventing file\nglob on empty prefix.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): chain after compsys command completion\n\n* fix(shell): restore chained gather fallback\n\n* fix(shell): preserve chained redraw after popup confirm\n\n* fix(shell): preserve chained redraw for single match\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-21T23:45:11+09:00",
          "tree_id": "4a8286f6d3d197216ec23969437c8c0ef31ab6de",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/bf39c8178e7876bd2b07164c0119e13781a3cb5e"
        },
        "date": 1774104800982,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 8018,
            "range": "± 817",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 74026,
            "range": "± 479",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 938131,
            "range": "± 48012",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 195238,
            "range": "± 9082",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 98536,
            "range": "± 5347",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 77060,
            "range": "± 845",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 15907,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/typo",
            "value": 28102,
            "range": "± 597",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 163714,
            "range": "± 4242",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11361,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/identical",
            "value": 279,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/transposition",
            "value": 141,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/substitution",
            "value": 280,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/long_strings",
            "value": 2163,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "damerau_levenshtein/different_len",
            "value": 207,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 101,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 35,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 136,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 856,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7570,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 749,
            "range": "± 14",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a1d87aa1ce647a99a6955798cf8c3bbaf630504e",
          "message": "perf(fuzzy): incremental filtering and cached query results (#10)\n\n* feat(fuzzy): always run Damerau-Levenshtein alongside nucleo matching\n\nThe DL fallback previously only fired when nucleo returned zero results.\nIn practice, nucleo almost always matched something via subsequence,\nso typos like `calude` → `claude` were never corrected.\n\nNow DL runs unconditionally (for queries >= 2 chars) and its novel\nresults are appended after nucleo matches with HashSet deduplication.\nSmart case (case-insensitive when query is all lowercase) is applied\nto DL via eq_ignore_ascii_case, avoiding per-candidate to_lowercase()\nheap allocations by reusing char buffers.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* perf(fuzzy): reuse scratch buffers\n\n* perf(app): incrementally narrow fuzzy results\n\n* perf(app): cache query results for backspace\n\n* fix(fuzzy): rerank typo fallback matches\n\n* fix(fuzzy): resolve clippy derivable_impls and needless_range_loop warnings\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(fuzzy): skip typo fallback for exact matches\n\n* fix(fuzzy): skip dl fallback after smart exact hits\n\n* fix(fuzzy): make dl smart-case unicode-aware\n\n* refactor(fuzzy): make typo matching fallback-only\n\n* test(bench): add unicode fuzzy benchmarks\n\n* fix: resolve clippy warnings\n\n* fix(app): preserve filtering after public state changes\n\n* Revert \"fix(app): preserve filtering after public state changes\"\n\nThis reverts commit 21906eff00b11104a03c7a0256c09a552d27069b.\n\n* refactor(fuzzy): delegate filter() to filter_matches()\n\nEliminate ~70 lines of duplicated logic. filter() now delegates to\nfilter_matches() and maps ScoredMatch → ScoredCandidate, removing\nthe separate damerau_levenshtein_fallback_candidates(),\nsort_empty_query_results(), and sort_scored_results() functions.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(app): note unbounded cached_filters growth\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(fuzzy): remove typo correction fallback\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-22T14:38:48+09:00",
          "tree_id": "24301ba7772ab3aa6793671c7e40e14571af6b9b",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/a1d87aa1ce647a99a6955798cf8c3bbaf630504e"
        },
        "date": 1774158491043,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7445,
            "range": "± 653",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 69195,
            "range": "± 475",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 907225,
            "range": "± 48058",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 191570,
            "range": "± 5866",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100788,
            "range": "± 2991",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 75017,
            "range": "± 838",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20063,
            "range": "± 1763",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18844,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 12055,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310877,
            "range": "± 2425",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 305434,
            "range": "± 1831",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 266642,
            "range": "± 1701",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 294638,
            "range": "± 3205",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26671,
            "range": "± 470",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 305525,
            "range": "± 6927",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3335948,
            "range": "± 28520",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 150883,
            "range": "± 2941",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 111862,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 361365,
            "range": "± 3465",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 729,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 114,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 39,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 90,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 71,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 856,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7576,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 747,
            "range": "± 4",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "72bd222a68dfc78a31d2cd0bc8284a5d07691fbc",
          "message": "fix: stabilize popup handoff reuse (#17)\n\n* fix(shell): smooth tab popup handoff\n\n* fix(shell): align cached fallback prefix length\n\n* fix(daemon): gate popup frame reuse\n\n* fix(shell): invalidate popup snapshot on full buffer changes\n\n* fix(shell): refresh cursor before popup reuse\n\n* fix(shell): clear popup state before subprocess fallback\n\n* fix(shell): degrade cleanly on reuse protocol mismatch\n\n* fix(shell): recompute candidates before popup reuse\n\n* fix(shell): validate geometry before popup reuse\n\n* fix(shell): restore flicker-free popup handoff\n\n* fix(daemon): make popup handoff deterministic\n\n* fix(daemon): redraw prompt before popup reuse\n\n* fix(shell): require handoff token for popup reuse\n\n* docs(handoff): explain why reuse token avoids zero\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(daemon): explain three-way initial frame decision\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(daemon): remove legacy_visible reuse path\n\nThe shell plugin already sends reuse_token=N exclusively and never\nsends reuse=1, so the legacy_visible fallback was dead code.\n\nAlso renames tracing field reuse_visible to reuse_requested for clarity.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): invalidate popup snapshot on terminal resize\n\nTRAPWINCH already clears the popup, but there is a race where Tab\nmay be pressed before the signal is delivered.  Guard against this\nby recording COLUMNS/LINES in the snapshot and checking them before\nreuse.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-22T20:37:54+09:00",
          "tree_id": "2f2c633801f153612d739f1fe42e9f924954894f",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/72bd222a68dfc78a31d2cd0bc8284a5d07691fbc"
        },
        "date": 1774180026730,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7242,
            "range": "± 809",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 73826,
            "range": "± 519",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 902845,
            "range": "± 14816",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 190299,
            "range": "± 1314",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100605,
            "range": "± 927",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 74772,
            "range": "± 2548",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 19949,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18823,
            "range": "± 1001",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 12037,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 311097,
            "range": "± 1546",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 305127,
            "range": "± 5113",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 266920,
            "range": "± 1071",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 296384,
            "range": "± 8482",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26662,
            "range": "± 350",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 304952,
            "range": "± 3761",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3341550,
            "range": "± 48593",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 150360,
            "range": "± 4661",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 111514,
            "range": "± 2165",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 354857,
            "range": "± 4647",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 740,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 114,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 34,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 90,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 49,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 858,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7573,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 747,
            "range": "± 7",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bfc137c9bac6004c190c03a389034f0f7bdd2622",
          "message": "fix(shell): prefer zsh/net/socket over deprecated zsh/net/unix (#20)\n\nzsh/net/unix is no longer shipped on some distros (e.g. Arch Linux)\nthat only provide zsh/net/socket. Both modules expose the same\nzsocket builtin for Unix domain sockets, but zsh/net/socket is the\ndocumented replacement in zsh 5.9+.\n\nTry zsh/net/socket first, falling back to zsh/net/unix for older\nenvironments. This fixes daemon startup on Arch and similar distros.\n\nCloses #19\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-23T14:16:36+09:00",
          "tree_id": "5bb59605cc90e7e6e1b02eacde151886b9896071",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/bfc137c9bac6004c190c03a389034f0f7bdd2622"
        },
        "date": 1774243547739,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 6738,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 74143,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 860851,
            "range": "± 33933",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 202974,
            "range": "± 827",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 103849,
            "range": "± 4770",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 74997,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 18706,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 17153,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 10713,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 308220,
            "range": "± 7556",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 301167,
            "range": "± 7656",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 263037,
            "range": "± 6234",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 292157,
            "range": "± 6632",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 25598,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 290430,
            "range": "± 1921",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3193843,
            "range": "± 18869",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 140761,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 105973,
            "range": "± 1078",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 348551,
            "range": "± 1694",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 589,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 36,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 105,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 92,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 44,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 57,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 64,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 121,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 817,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7409,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 737,
            "range": "± 15",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "105966658+oyoshot@users.noreply.github.com",
            "name": "oyoshot",
            "username": "oyoshot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6e2c2d87ed95ccc3e009410488dda02385d59d97",
          "message": "fix(ui): skip popup render when filtered candidates are empty (#13) (#21)\n\nGuard all four render/complete entry points so that an empty\nfiltered_indices after App::new bails out before drawing.  This\nprevents the border-only popup that appeared when the shell reused\ncached candidates whose fuzzy filter produced no matches.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-23T14:46:44+09:00",
          "tree_id": "70399e7bf0dfd356100687a3f0035060aebd5616",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/6e2c2d87ed95ccc3e009410488dda02385d59d97"
        },
        "date": 1774245365555,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7334,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 74542,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 905108,
            "range": "± 56235",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 191730,
            "range": "± 1591",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100815,
            "range": "± 1060",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 75110,
            "range": "± 988",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20200,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18887,
            "range": "± 340",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 12479,
            "range": "± 949",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 315170,
            "range": "± 2517",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 309248,
            "range": "± 14025",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 270738,
            "range": "± 2017",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 299734,
            "range": "± 12092",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26797,
            "range": "± 488",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 304446,
            "range": "± 3112",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3343766,
            "range": "± 22044",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 150521,
            "range": "± 6867",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 112050,
            "range": "± 978",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 359292,
            "range": "± 5862",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 736,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 41,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 114,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 95,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 38,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 51,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 65,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 70,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 135,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 853,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7589,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 739,
            "range": "± 7",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}