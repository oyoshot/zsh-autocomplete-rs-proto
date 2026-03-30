window.BENCHMARK_DATA = {
  "lastUpdate": 1774887984464,
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
          "id": "a70cf099f56c67c18df647411be010b41e12576b",
          "message": "fix(ui): defer popup highlight until user navigates (#22)\n\n* fix(ui): defer popup highlight until user navigates (#14)\n\nChange App.selected from usize to Option<usize> so the popup opens\nwith no candidate highlighted.  None means the user has not navigated\nyet; the first Down arrow activates Some(0) and Up activates the last\nitem.  Confirm with no selection now returns the filter text instead\nof an empty string, matching Cancel semantics.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(ui): return filter_text on confirm with no selection\n\nWhen the user presses Enter without navigating, the direct-execution\npath now returns the current filter text (exit code 1) instead of\nempty output, matching the daemon Confirm-None and Cancel semantics.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(app): encapsulate selected field behind accessor methods\n\nMake the selected field private and expose selected() getter and\nset_selected() (test-only) setter.  All test code now uses these\nmethods instead of direct field access.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-23T15:29:04+09:00",
          "tree_id": "69b6f57d23d8d62831d3f6ce1d85b23f4cd42df2",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/a70cf099f56c67c18df647411be010b41e12576b"
        },
        "date": 1774247905263,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7353,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75516,
            "range": "± 943",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 917052,
            "range": "± 62152",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 192308,
            "range": "± 5391",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 101392,
            "range": "± 2124",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76770,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20220,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19071,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11846,
            "range": "± 702",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310239,
            "range": "± 1512",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 303309,
            "range": "± 2707",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 265135,
            "range": "± 1808",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 299217,
            "range": "± 7915",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26512,
            "range": "± 483",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 302508,
            "range": "± 1730",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3319937,
            "range": "± 16022",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154565,
            "range": "± 1336",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 115162,
            "range": "± 6842",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 369951,
            "range": "± 38472",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 732,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 41,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 114,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 33,
            "range": "± 2",
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
            "range": "± 7",
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
            "range": "± 4",
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
            "range": "± 1",
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
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7588,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 746,
            "range": "± 8",
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
          "id": "36491eb42245eab74563dafb091ee23b7b929f0c",
          "message": "fix(ui): auto-select first candidate in interactive complete mode (#24)\n\n* fix(ui): auto-select first candidate in interactive complete mode (#23)\n\nAfter #22 changed `selected` to `None` by default, entering interactive\ncomplete mode via Tab no longer highlighted the first candidate until\na second keypress. Additionally, filtering (type_char / backspace)\nreset the selection back to None, losing the highlight.\n\nFix by calling `app.move_down()` at three points in the complete path:\n  - before the initial frame (both run_complete and handle_complete)\n  - after type_char filtering\n  - after backspace filtering\n\nThe daemon's initial-frame reuse optimisation (NONE / prompt-patch) and\nthe associated `CompleteReuse` type are removed because the highlight\nstate always differs from the prior render frame. The render path\nremains unchanged so #14's deferred highlight is preserved.\n\nCloses #23\nCloses #25\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(ui): reselect first complete candidate after filtering\n\n* fix(ui): confirm top complete candidate after filtering\n\n* fix(ui): keep complete selection visible after filtering\n\n* fix(ui): preserve typed text after complete filtering\n\n* fix(ui): preserve common-prefix confirm in complete mode\n\n* fix(daemon): preserve reused popup highlighting\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-23T19:05:21+09:00",
          "tree_id": "97dd6a72d266c259ca593abf889583fd260336ae",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/36491eb42245eab74563dafb091ee23b7b929f0c"
        },
        "date": 1774260915439,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7424,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75369,
            "range": "± 1001",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 916364,
            "range": "± 75830",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 190112,
            "range": "± 2818",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 101361,
            "range": "± 959",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76005,
            "range": "± 7677",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20128,
            "range": "± 1172",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19076,
            "range": "± 509",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11797,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310634,
            "range": "± 9657",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 303614,
            "range": "± 5250",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 265793,
            "range": "± 19911",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 293254,
            "range": "± 2049",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26583,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 304412,
            "range": "± 1356",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3324131,
            "range": "± 231325",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154469,
            "range": "± 1847",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114215,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 364848,
            "range": "± 4448",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 743,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 42,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 114,
            "range": "± 0",
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
            "value": 96,
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
            "value": 28,
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
            "value": 855,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7583,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 747,
            "range": "± 5",
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
          "id": "b72138d67468f662aeaab4468253a3cf884f5413",
          "message": "fix(shell): prevent fast typing from dropping characters (#28)\n\n* fix(shell): prevent fast typing from dropping characters\n\nRemove the `read -t 0` tty buffer flush in `_zacrs_get_cursor_pos`\nthat consumed pending user keystrokes before the DSR cursor query.\n\nAdd two `PENDING` guards in `_zacrs_line_pre_redraw` so that heavy\nwork (compsys, gather, render) is skipped while the user is still\ntyping. The next `line-pre-redraw` fires after ZLE processes the\nqueued keystrokes and retries with `PENDING == 0`.\n\nCloses #27\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): query cursor position before compsys in Tab path\n\nMove the DSR cursor query to the start of _zacrs_tab_complete,\nbefore the heavy compsys/gather phase.  Right after ZLE processes\nthe Tab key the tty input buffer is empty, so the query always\ngets a clean response.  Keys typed while compsys runs stay in the\nbuffer and are later consumed naturally by the daemon interactive\nloop (raw-mode /dev/tty read), preserving type-ahead.\n\nThis eliminates the need for the tty buffer flush in\n_zacrs_get_cursor_pos: every call site now guarantees an empty\nbuffer (auto-trigger via PENDING guards, Tab via early query).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): address review — revert early DSR, fix PENDING retry\n\nRevert the early DSR query added in the previous commit: moving\n_zacrs_get_cursor_pos to the top of _zacrs_tab_complete forced every\nnon-reuse Tab press to pay the DSR cost (up to 1 s timeout on\nterminals that don't respond), even for no-candidate or single-\ncandidate paths that never open a popup.  The drain in\nget_cursor_pos is restored so the Tab path keeps a clean DSR\nresponse (matching pre-fix behaviour).\n\nFix the PENDING guard state management so skipped buffers are\nalways retried on the next redraw:\n\n  - Guard #1 (before compsys): move _zacrs_prev_lbuffer update to\n    AFTER the PENDING check.  A PENDING-skip now leaves prev_lbuffer\n    at its old value, so the next line-pre-redraw still sees a\n    changed LBUFFER and re-enters the processing path.\n\n  - Guard #2 (before render): reset _zacrs_prev_lbuffer to \"\" before\n    returning.  This forces the next redraw to retry even if the\n    queued input didn't change LBUFFER (e.g. arrow keys, Home/End).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): replace DSR flush with robust byte-by-byte parser\n\nReplace the `read -t 0` tty flush and `read -d R` DSR reader with\na byte-by-byte loop that matches the full `\\e[row;colR` pattern.\n\nThe old approach had two problems:\n  - The flush explicitly discarded buffered keystrokes\n  - `read -d R` stopped at the first 'R' in the buffer, which\n    could be a user keystroke rather than the DSR terminator\n\nThe new parser reads one byte at a time and applies a regex match\nafter each byte.  It correctly handles stale keystrokes in the tty\nbuffer even when they contain 'R' or '['.  Any bytes preceding the\nESC are consumed as a side-effect of reading through the shared tty\nbuffer — this is inherent to the single-buffer tty design and the\nauto-trigger path avoids it entirely via PENDING guards.\n\nA 256-byte safety limit prevents unbounded reads if no DSR response\narrives.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): recover Tab type-ahead via daemon KEY injection\n\n_zacrs_get_cursor_pos now saves any pre-DSR keystrokes (bytes that\npreceded the \\e[row;colR response) in the global _zacrs_cursor_stale\ninstead of silently discarding them.\n\n_zacrs_invoke_daemon re-injects the saved keystrokes into the daemon\ninteractive loop as individual KEY commands, right after raw-mode\nentry and before the normal key-read loop.  The daemon processes\nthem as filter input, so typing \"st\" during compsys immediately\nfilters the popup to matching candidates.  If the daemon sends DONE\nduring injection (e.g. all candidates filtered away), the main loop\nis skipped via an _inject_done flag.\n\nThe subprocess fallback (_zacrs_invoke) and the auto-trigger path\n(_zacrs_render) clear _zacrs_cursor_stale without injection, since\nthey have no interactive loop (subprocess) or are guarded by PENDING\nchecks (auto-trigger) respectively.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): recover ESC-prefixed type-ahead and subprocess fallback\n\nImprove type-ahead recovery in two areas flagged by review:\n\n1. ESC-prefixed keystrokes (arrows, Home, End, Alt-...):\n   Use MBEGIN from the DSR regex match to capture ALL bytes before\n   the DSR response, not just those before the first ESC.  The\n   injection loop now groups ESC-prefixed sequences (scanning until\n   a terminating letter or ~) into single KEY commands, matching\n   the main interactive loop's behaviour.\n\n2. Subprocess fallback (_zacrs_invoke):\n   Push stale bytes back to ZLE via `zle -U` so they are processed\n   after the widget returns, instead of silently discarding them.\n   The daemon path remains the preferred recovery mechanism (direct\n   KEY injection into the interactive loop).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): recover stale bytes on all invoke_daemon early-return paths\n\nThe initial_done branch (daemon sends DONE before the interactive\nloop) and the error branches (NONE without reuse, unknown response)\nall returned without handling _zacrs_cursor_stale.  Push the saved\nkeystrokes back to ZLE via `zle -U` and clear the global on every\nearly-return path that follows _zacrs_get_cursor_pos, ensuring no\ntype-ahead is silently lost and no stale data leaks to the next\ninvocation.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): recover stale bytes on zsocket connect failure\n\nWhen zsocket fails after _zacrs_get_cursor_pos has already consumed\ntype-ahead into _zacrs_cursor_stale, push the bytes back to ZLE\nand clear the global before returning.  The caller falls back to\n_zacrs_invoke with pre-computed cursor coordinates, so its own\nzle -U branch is never reached — this was the last uncovered\nearly-return path.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(shell): extract response handler from daemon interactive loops\n\nDeduplicate FRAME/DONE/NONE response handling in the injection and\nmain loops of _zacrs_invoke_daemon into a shared helper function\n_zacrs_complete_handle_response.  Also replace per-call `local -a`\narray allocation in the DONE branch with inline parameter expansion.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-24T00:59:19+09:00",
          "tree_id": "3c48aa39be867999da7cd508aa1e5d210f85190f",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/b72138d67468f662aeaab4468253a3cf884f5413"
        },
        "date": 1774282115542,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7376,
            "range": "± 836",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75653,
            "range": "± 504",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 918047,
            "range": "± 39198",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 193143,
            "range": "± 2096",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 101896,
            "range": "± 603",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76512,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20238,
            "range": "± 422",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19208,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11836,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 313679,
            "range": "± 4588",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 306529,
            "range": "± 3642",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 269327,
            "range": "± 2769",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 296185,
            "range": "± 3192",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 27260,
            "range": "± 807",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 303823,
            "range": "± 2816",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3325619,
            "range": "± 22955",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154641,
            "range": "± 1938",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114881,
            "range": "± 2038",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 369382,
            "range": "± 7202",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 731,
            "range": "± 13",
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
            "value": 115,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 34,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 96,
            "range": "± 1",
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
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 50,
            "range": "± 2",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 135,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 857,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7601,
            "range": "± 886",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 750,
            "range": "± 11",
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
          "id": "874eb2dfc60a36041ed444bf312f43a7379af098",
          "message": "fix(core): panic on multi-byte UTF-8 filenames in compute_common_prefix (#31)\n\n* fix(core): panic on multi-byte UTF-8 filenames in compute_common_prefix\n\nSnap byte-level common-prefix length back to a valid char boundary\nbefore slicing, preventing panics on CJK, emoji, and accented\nfilenames.\n\nCloses #30\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* style: apply rustfmt to multibyte test cases\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-25T10:11:54+09:00",
          "tree_id": "fa5732cb2ba67302f01304bbe308634a1ac044c9",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/874eb2dfc60a36041ed444bf312f43a7379af098"
        },
        "date": 1774401663471,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7507,
            "range": "± 550",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 71122,
            "range": "± 833",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 915331,
            "range": "± 61491",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 192933,
            "range": "± 1517",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 103534,
            "range": "± 1319",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76311,
            "range": "± 2124",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20215,
            "range": "± 1338",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19119,
            "range": "± 716",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11802,
            "range": "± 129",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310388,
            "range": "± 1576",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 303845,
            "range": "± 4426",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 265691,
            "range": "± 2158",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 292859,
            "range": "± 13705",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 27328,
            "range": "± 907",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 306689,
            "range": "± 6869",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3348928,
            "range": "± 119085",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154143,
            "range": "± 1447",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114489,
            "range": "± 1657",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 367601,
            "range": "± 4089",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 751,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 41,
            "range": "± 5",
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
            "value": 33,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 96,
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
            "value": 91,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 28,
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
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 135,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 852,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7495,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 754,
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
          "id": "0d778df5d63cfd9a765695b709f0c4a092433a3c",
          "message": "fix(shell): prevent reuse of gather-derived popup on Tab (#33)\n\n* fix(shell): prevent reuse of gather-derived popup on Tab (#16)\n\nWhen auto-trigger built a popup from _zacrs_gather (heuristic fallback),\npressing Tab reused those candidates without running compsys, which could\ninsert incorrect text missing compsys-side formatting (quoting, suffix\nhandling, compadd -P/-S/-p transformations).\n\nTrack candidate provenance via from_gather flag through the snapshot and\ncache layers. Gather-derived popups are no longer reused on Tab —\ncompsys runs fresh, ensuring correct completion semantics. Compsys-derived\npopups continue to be reused flicker-free.\n\nAlso extract _zacrs_reset_cache helper to consolidate the three-site\ncache invalidation pattern.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): clear stale popup rows on daemon Tab redraw\n\nWhen the gather-popup reuse guard forces a fresh daemon FRAME,\nthe previous (taller) popup's bottom rows could remain on screen.\n\nClear only the old rows that the new frame won't overwrite, right\nbefore sysread draws the new popup.  This keeps the gap invisible\nand avoids the flicker that a full _zacrs_clear_popup before the\ndaemon call would introduce.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-25T13:33:29+09:00",
          "tree_id": "cabe95ed10427d4a1c8813b40de358e615c0f70e",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/0d778df5d63cfd9a765695b709f0c4a092433a3c"
        },
        "date": 1774413767450,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7435,
            "range": "± 677",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75281,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 923445,
            "range": "± 37933",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 192615,
            "range": "± 2311",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 101181,
            "range": "± 2065",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76388,
            "range": "± 6815",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20164,
            "range": "± 263",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19090,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11809,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310328,
            "range": "± 3403",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 304084,
            "range": "± 11008",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 266391,
            "range": "± 1270",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 299657,
            "range": "± 8852",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26538,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 303463,
            "range": "± 26711",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3317277,
            "range": "± 35438",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154322,
            "range": "± 1660",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114580,
            "range": "± 751",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 364118,
            "range": "± 9688",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 743,
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
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_no_trunc",
            "value": 33,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/cjk_trunc",
            "value": 95,
            "range": "± 2",
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
            "value": 50,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 65,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 71,
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
            "value": 847,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7479,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 744,
            "range": "± 56",
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
          "id": "afa74d942dfecf7e693a04334b5be77abfac7901",
          "message": "chore: remove initial design docs in favor of issue tracking\n\nDesign notes and architecture analysis in docs/ were written as\ninitial Coding Agent context. Now that decisions and problems are\ntracked via GitHub Issues, these files are redundant.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-25T14:25:14+09:00",
          "tree_id": "3e975c498a8478c5529bc63169eb83896c1a0f57",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/afa74d942dfecf7e693a04334b5be77abfac7901"
        },
        "date": 1774416783986,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7576,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 76652,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 921501,
            "range": "± 11594",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 193108,
            "range": "± 2073",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 101399,
            "range": "± 1436",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 77011,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20096,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19053,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11795,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 309617,
            "range": "± 1028",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 302555,
            "range": "± 1198",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 265175,
            "range": "± 5485",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 292301,
            "range": "± 1821",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26479,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 302835,
            "range": "± 1495",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3313052,
            "range": "± 10119",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 156957,
            "range": "± 2970",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114229,
            "range": "± 1082",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 369024,
            "range": "± 2481",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 743,
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
            "range": "± 0",
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
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 28,
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
            "value": 845,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7488,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 743,
            "range": "± 1",
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
          "id": "aa09fd828815eea72fd5bbbdf34322710c5eaf5e",
          "message": "fix(ui): auto-select first candidate regardless of common prefix length (#29) (#35)\n\nRemove the `filter_text == prefix` guard before `select_first()` in both\nthe subprocess and daemon complete paths. When the longest common prefix\nexceeds the typed prefix (e.g. typing \"car\" with candidates \"cargo\",\n\"cargo-add\"), the guard prevented auto-selection, requiring a second Tab.\n\nThe render path (auto-trigger) is unaffected as it never calls\n`select_first()`.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-25T15:02:10+09:00",
          "tree_id": "1735d969f01ba2e1202782ad585b8f1baf415360",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/aa09fd828815eea72fd5bbbdf34322710c5eaf5e"
        },
        "date": 1774418987265,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7361,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75283,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 913077,
            "range": "± 13786",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 192977,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100601,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76084,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20132,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19079,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11794,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 309361,
            "range": "± 779",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 302937,
            "range": "± 2150",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 265385,
            "range": "± 1009",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 292734,
            "range": "± 1830",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26486,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 302902,
            "range": "± 1012",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3310895,
            "range": "± 54280",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154015,
            "range": "± 2192",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114532,
            "range": "± 874",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 368021,
            "range": "± 2712",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 738,
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
            "range": "± 0",
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
            "value": 96,
            "range": "± 0",
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
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 134,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 846,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7478,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 743,
            "range": "± 16",
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
          "id": "fe67f40345b95f3977140b614a0455ecd8163c22",
          "message": "feat(shell): add Tab-cycle completion mode (#18) (#36)\n\n* feat(shell): add Tab-cycle completion mode (#18)\n\nReplace blocking interactive loop with lightweight cycle-mode keymap.\nTab cycles through candidates with highlight-only updates (no LBUFFER\nmodification until accept/exit), cursor hiding during atomic\nclear+render to eliminate flicker, and cached cursor position to\navoid repeated DSR queries.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor: deduplicate shell helpers and extract Popup::format_metadata\n\nExtract shared logic from cycle and interactive completion paths:\n- _zacrs_parse_render_header: unifies 3 header parsing loops\n- _zacrs_collect_candidates: unifies candidate collection (compsys/gather/cache)\n- _zacrs_apply_single_candidate: unifies single-candidate immediate completion\n- Popup::format_metadata: consolidates metadata formatting from daemon.rs and main.rs\n\nAlso: use _zacrs_cycle_exit() in accept_line/send_break, remove dead\n_zacrs_cycle_pending_render variable, clear cycle state on exit.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): handle unbound keys during Tab-cycle mode\n\n- Bind Backspace (^? / ^H) to cancel in the cycle keymap so it\n  doesn't silently modify LBUFFER via the inherited main binding\n- Add a safety net in line-pre-redraw: if an unhandled key (e.g.\n  multi-byte character input) mutates LBUFFER during cycle mode,\n  auto-exit cycle and fall through to the normal auto-trigger flow\n- Skip the \"OK\" daemon header prefix in _zacrs_parse_render_header\n  so it never transiently sets _zacrs_parsed_tty_len to a non-numeric\n- Use saturating_sub in App::move_up() to guard against a theoretical\n  panic if max_visible were ever 0\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): eliminate Tab-cycle popup flicker with selective clear\n\nReplace blanket clear of all previous popup rows with selective clear\nthat only erases rows not covered by the new popup. Also switch from\nhead -c (fork+exec) to sysread builtin and drop redundant cursor::Show.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): eliminate auto-trigger popup flicker and cursor jump\n\nDefer popup clear in line-pre-redraw so _zacrs_render can batch\nold-popup clear + new-popup draw in a single output group with cursor\nhidden. This prevents the visible blank frame between clear and draw.\n\n- _zacrs_clear_popup: add cursor hide/show, group writes via { } > /dev/tty\n- _zacrs_render daemon path: save old popup geometry, clear all old rows\n  atomically before drawing new popup; replace head -c with sysread\n- _zacrs_line_pre_redraw: defer clear to render, add explicit clear at\n  early-return points where no new popup will be drawn\n- Error/EMPTY/socket-fail paths: clear stale popup properly\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): add sysread error handling to cycle render path\n\nMirror the tty_ok guard from _zacrs_render into\n_zacrs_cycle_render_and_apply so a daemon crash mid-response correctly\nclears the popup and marks the daemon unavailable instead of leaving\n_zacrs_popup_visible=1 with nothing drawn.\n\nAlso add a sync comment noting the two daemon paths must stay aligned.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): exit cycle mode immediately on EMPTY daemon response\n\nWhen the daemon returns EMPTY during Tab-cycle (no candidates match the\nfilter), exit cycle mode right away instead of falling through to the\nsubprocess fallback which would produce the same EMPTY result.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(shell): simplify redundant parameter expansion in apply_single_candidate\n\nRemove unnecessary outer ${} wrapping around the ##-expansion for\nextracting the candidate kind field.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): use _zacrs_cycle_exit in cycle render fallback path\n\nThe subprocess fallback in _zacrs_cycle_render_and_apply was manually\nresetting only _zacrs_cycle_active and the keymap, missing state cleanup\n(prev_lbuffer, candidates, prefix, original_lbuffer). Replace with\n_zacrs_cycle_exit which handles all cleanup consistently.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* test(ui): add unit test for Popup::format_metadata\n\nVerify the exact key=value format that the shell parser\n_zacrs_parse_render_header depends on, both with and without\nselected_original_idx.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): restore cursor visibility after render paths\n\nBoth _zacrs_render and _zacrs_cycle_render_and_apply hide the cursor\nwith \\e[?25l but never restore it with \\e[?25h. If sysread fails or\ntty_len is 0, the cursor stays invisible. Add cursor restore at the\nend of both atomic output blocks.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(shell,daemon): use tty_len key-value in render header\n\nReplace the fragile bare-number detection for tty_len in\n_zacrs_parse_render_header with an explicit tty_len=N key-value pair,\nconsistent with all other header fields. The daemon text protocol now\nemits \"OK ... tty_len=N\" instead of appending a bare number.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(protocol): remove old-client backward compat for flags byte\n\nAlways expect the flags byte in binary protocol Render requests.\nReturn a parse error instead of silently defaulting to None when the\nflags byte is missing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(daemon): introduce RenderParams to reduce handle_render args\n\nReplace 6 individual parameters with a RenderParams struct, removing\nthe need for #[allow(clippy::too_many_arguments)].\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(shell): clarify index mapping in cycle_apply_selected\n\nAdd a comment explaining that selected_original_idx maps into the\nRust-side all_candidates array, which corresponds 1:1 with the shell\ncands array.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): guard against empty sel_line in cycle_apply_selected\n\nIf selected_original_idx is out of range for the candidates array,\nsel_line becomes empty and LBUFFER would be truncated to just the\nbase prefix. Add an early return guard to preserve LBUFFER unchanged.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): move tty_ok declaration before output group\n\nMove `local tty_ok=1` outside the `{ } > /dev/tty` block in\n_zacrs_cycle_render_and_apply so the variable scope is visually\nclear.  zsh does not create a subshell for `{ }`, so behavior is\nunchanged, but declaring it before the block makes intent explicit.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(protocol): use safe u16 conversion for selected index\n\nReplace `selected.map(|s| s as u16)` with `and_then(try_from.ok())`\nto avoid silent truncation when selected index exceeds u16::MAX.\nOverflow now yields None (no selection), which is safe because\napp.set_selected already bounds-checks against filtered_indices.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(shell): rename cycle_render_and_apply to cycle_render_selected\n\nThe function only renders the popup with a highlighted selection —\nit does not modify LBUFFER.  Rename to _zacrs_cycle_render_selected\nand update the docstring to match the actual behavior.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(shell): document multi-byte input behavior in cycle keymap\n\nExplain that unbound keys in the _zacrs_cycle keymap (copied from\nmain) fall through to self-insert, and the line-pre-redraw hook\ndetects the resulting LBUFFER mutation to auto-exit cycle mode.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(shell): extract _zacrs_daemon_send_render to reduce duplication\n\nMove the shared daemon connection logic (zsocket connect, send render\nrequest, read+parse response header) into a single helper function.\nBoth _zacrs_render and _zacrs_cycle_render_selected now call the\nhelper and only handle the tty draw phase and error paths themselves,\nwhich differ between auto-trigger (full clear) and cycle (selective\nclear) modes.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(shell): extract _zacrs_daemon_draw_atomic to reduce render duplication\n\nConsolidate the atomic clear+draw output groups from _zacrs_render and\n_zacrs_cycle_render_selected into a shared helper. The selective flag\ncontrols whether all previous rows are cleared (auto-trigger) or only\nrows outside the new popup region (cycle mode).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(daemon): use safe usize::from instead of as usize for selected\n\nConsistent with the safe conversion approach established in 357176e.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-26T15:33:08+09:00",
          "tree_id": "00b42ec7e50d3345fe51f3d5b775c7a6a449db82",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/fe67f40345b95f3977140b614a0455ecd8163c22"
        },
        "date": 1774507247414,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7412,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75423,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 910188,
            "range": "± 10721",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 191411,
            "range": "± 2543",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100219,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76104,
            "range": "± 610",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20136,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19099,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11809,
            "range": "± 576",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310063,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 303359,
            "range": "± 1550",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 266041,
            "range": "± 1490",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 293364,
            "range": "± 1697",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26547,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 303324,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3317922,
            "range": "± 9202",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 154530,
            "range": "± 791",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114549,
            "range": "± 369",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 365172,
            "range": "± 2330",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 734,
            "range": "± 13",
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
            "range": "± 0",
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
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 90,
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
            "value": 861,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7584,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 743,
            "range": "± 2",
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
          "id": "3c53619ea08c12510b63b9c2e358a4897f15664d",
          "message": "fix(shell): enable chained completion popup after cycle-mode Space confirm (#37)\n\n_zacrs_cycle_accept_space had two bugs preventing chained completion:\n1. _zacrs_suppressed=1 caused line-pre-redraw to return early before\n   reaching the chain_retry logic\n2. _zacrs_cycle_exit set prev_lbuffer=\"$LBUFFER\", making line-pre-redraw\n   think nothing changed\n\nRemove suppressed flag and override prev_lbuffer=\"\" so the auto-trigger\nhook detects the buffer change and fires compsys for next-level candidates\n(e.g. subcommands after completing a command name).\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-26T15:58:41+09:00",
          "tree_id": "e1c99c3bc1b5c03f63a65586e591368977cf033d",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/3c53619ea08c12510b63b9c2e358a4897f15664d"
        },
        "date": 1774508769950,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7423,
            "range": "± 101",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75883,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 899387,
            "range": "± 9734",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 193922,
            "range": "± 2488",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 102415,
            "range": "± 388",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76399,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20134,
            "range": "± 537",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19113,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11802,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 310363,
            "range": "± 1390",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 304125,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 266267,
            "range": "± 1279",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 293400,
            "range": "± 1860",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26643,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 304084,
            "range": "± 2034",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3323858,
            "range": "± 8600",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 155343,
            "range": "± 1043",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 115463,
            "range": "± 2184",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 370263,
            "range": "± 6425",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 745,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 41,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 114,
            "range": "± 0",
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
            "value": 38,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_trunc",
            "value": 90,
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
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 845,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7481,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 747,
            "range": "± 3",
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
          "id": "4cf66356a51347351fe391ac5edd4bd0d76e9684",
          "message": "fix(shell): clear stale POSTDISPLAY on cycle mode exit (#38)\n\nWhen the Tab-cycle widget modified LBUFFER (e.g. \"car\" → \"cargo\"),\nPOSTDISPLAY set by other plugins remained stale, causing ghost text\nartifacts like \"cargo go install --path .\" instead of \"cargo install\n--path .\".  Unset POSTDISPLAY in _zacrs_cycle_exit so the old value\nis never carried over after the buffer changes.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-26T16:50:12+09:00",
          "tree_id": "264d89f7c7690a178e696ce462d04b224a6fdaa2",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/4cf66356a51347351fe391ac5edd4bd0d76e9684"
        },
        "date": 1774511870872,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7387,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75535,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 919296,
            "range": "± 12613",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 192403,
            "range": "± 927",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 101161,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76125,
            "range": "± 1441",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20141,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19089,
            "range": "± 459",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11795,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 320337,
            "range": "± 4748",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 323936,
            "range": "± 3580",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 275291,
            "range": "± 5172",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 302264,
            "range": "± 4695",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 28031,
            "range": "± 1182",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 320368,
            "range": "± 13118",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3502629,
            "range": "± 149257",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 153755,
            "range": "± 4865",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114417,
            "range": "± 1907",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 368213,
            "range": "± 2411",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 753,
            "range": "± 16",
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
            "range": "± 0",
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
            "value": 96,
            "range": "± 1",
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
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 28,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 135,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 846,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7462,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 743,
            "range": "± 3",
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
          "id": "e99aff9bedda9b17837ba861ec2fe9e3320183a1",
          "message": "refactor(daemon): deduplicate and restructure DaemonServer (#39)\n\n* refactor(daemon): extract cap_viewport_and_scroll helper\n\nDeduplicate the viewport capping and scroll-up computation logic\nthat was duplicated between handle_render and handle_complete.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(daemon): extract text protocol parsing helpers\n\nIntroduce parse_terminal_dims and read_prefix_and_tsv to deduplicate\nthe header parsing and body reading shared by the render and complete\nbranches in handle_text_connection.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(daemon): promote send_frame closure to method\n\nMove the send_frame closure out of handle_complete into a proper\nDaemonServer method, eliminating the captured theme/bindings bindings\nand making it independently testable.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(daemon): introduce CompleteParams struct\n\nReplace the 10-parameter handle_complete signature with a\nCompleteParams struct, removing the #[allow(clippy::too_many_arguments)]\nannotation and making call sites self-documenting.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-26T17:23:28+09:00",
          "tree_id": "4e2eb0d793db2fbd383c369a55fe431fe1839d59",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/e99aff9bedda9b17837ba861ec2fe9e3320183a1"
        },
        "date": 1774513866589,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7692,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75401,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 915160,
            "range": "± 16841",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 191314,
            "range": "± 2456",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100924,
            "range": "± 468",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 76181,
            "range": "± 380",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20146,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19072,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11791,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 309156,
            "range": "± 6392",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 302976,
            "range": "± 983",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 265565,
            "range": "± 1154",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 292911,
            "range": "± 1373",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26839,
            "range": "± 451",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 306193,
            "range": "± 3776",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3346177,
            "range": "± 26313",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 156041,
            "range": "± 1463",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 114178,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 375207,
            "range": "± 17623",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 732,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 28,
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
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 140,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 851,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7479,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 743,
            "range": "± 1",
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
          "id": "f3cd49bf2a539cc960854a62f38a9793d1449289",
          "message": "fix(shell): remove redundant clear calls that cause popup flicker (#40)\n\n* fix(shell): remove redundant _zacrs_clear_popup calls that cause flicker\n\nThe subprocess fallback in _zacrs_render was preceded by a clear in\nthe daemon-unavailable branch, causing a double erase before redraw.\nSimilarly, _zacrs_cycle_render_selected cleared the popup before\ncalling _zacrs_render which already clears internally. Remove both\nredundant calls so the popup is erased exactly once.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): buffer popup draw into single write() with sync output markers\n\nReplace the multi-printf `{ } > /dev/tty` pattern in _zacrs_daemon_draw_atomic\nand _zacrs_clear_popup with a single buffered printf. This reduces 5-10\nwrite() syscalls per render to exactly one, and embeds Synchronized Output\nmarkers (\\e[?2026h/l) inside the same write so compliant terminals render\nthe clear+draw atomically without nesting conflicts with ZSH's own sync\nregions.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* perf(shell): skip DSR cursor query when popup is already visible\n\nWhen the popup is on screen and the terminal has not resized, reuse the\ncursor position from the previous render instead of sending \\e[6n and\nreading the response byte-by-byte. This eliminates one /dev/tty write\nand the blocking read loop per keystroke, reducing render latency.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-26T23:24:01+09:00",
          "tree_id": "181a4dde94a23851a707948259470f71cb98ba63",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/f3cd49bf2a539cc960854a62f38a9793d1449289"
        },
        "date": 1774535505033,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7236,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 73822,
            "range": "± 676",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 901215,
            "range": "± 15359",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 193235,
            "range": "± 865",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100885,
            "range": "± 527",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 74686,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 19522,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18342,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11535,
            "range": "± 1202",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 333514,
            "range": "± 7624",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 328649,
            "range": "± 6705",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 290634,
            "range": "± 6960",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 319623,
            "range": "± 6676",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 27041,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 309363,
            "range": "± 1449",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3384633,
            "range": "± 20718",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 150259,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 112672,
            "range": "± 964",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 360240,
            "range": "± 7745",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 734,
            "range": "± 13",
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
            "value": 34,
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
            "value": 38,
            "range": "± 0",
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
            "value": 28,
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
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 72,
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
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7607,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 754,
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
          "id": "96ba47d0f41fa4d7da1bb4158366067973656547",
          "message": "chore(deps): bump toml from 1.0.7+spec-1.1.0 to 1.1.0+spec-1.1.0 (#42)\n\nBumps [toml](https://github.com/toml-rs/toml) from 1.0.7+spec-1.1.0 to 1.1.0+spec-1.1.0.\n- [Commits](https://github.com/toml-rs/toml/compare/toml-v1.0.7...toml-v1.1.0)\n\n---\nupdated-dependencies:\n- dependency-name: toml\n  dependency-version: 1.1.0+spec-1.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-27T14:46:38+09:00",
          "tree_id": "9c6fe016a54747adf0e7d401f19460498e1537f8",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/96ba47d0f41fa4d7da1bb4158366067973656547"
        },
        "date": 1774590852606,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7372,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 74199,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 894468,
            "range": "± 19659",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 194242,
            "range": "± 6243",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100645,
            "range": "± 654",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 74777,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20062,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18836,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11983,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 307302,
            "range": "± 17780",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 301646,
            "range": "± 4249",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 263253,
            "range": "± 9868",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 291516,
            "range": "± 5173",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26323,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 301525,
            "range": "± 955",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3296152,
            "range": "± 26954",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 152181,
            "range": "± 849",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 112928,
            "range": "± 3440",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 360789,
            "range": "± 7481",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 720,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 112,
            "range": "± 4",
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
            "value": 96,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/mixed_no_trunc",
            "value": 39,
            "range": "± 1",
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
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 49,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 64,
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
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 867,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7609,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 756,
            "range": "± 3",
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
          "id": "efe0ee00937c51cccee43a830c0840b55116c1ed",
          "message": "chore: enable LTO and single codegen-unit for bench profile (#45)\n\nMatch the release profile settings so benchmarks measure\nperformance closer to the shipped binary.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-30T15:17:05+09:00",
          "tree_id": "219c69a714ada4151aa0d814ff7203309ffb32d6",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/efe0ee00937c51cccee43a830c0840b55116c1ed"
        },
        "date": 1774851880435,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7256,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 74114,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 895791,
            "range": "± 16990",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 190273,
            "range": "± 2345",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100699,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 75077,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 19936,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18832,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11983,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 306791,
            "range": "± 2659",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 300583,
            "range": "± 3906",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 262324,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 290615,
            "range": "± 1470",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26344,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 301267,
            "range": "± 3693",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3293694,
            "range": "± 34253",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 150914,
            "range": "± 2052",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 112891,
            "range": "± 548",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 360230,
            "range": "± 2694",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 730,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 112,
            "range": "± 0",
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
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 64,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 857,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7592,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 756,
            "range": "± 1",
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
          "id": "41c9d587b54067a96e1b08b5a16bbb8248099ab4",
          "message": "refactor(input): move popup key handling from Zsh to Rust (#43)\n\n* feat(daemon): move cycle-mode key handling from Zsh to Rust (#41)\n\nAdd KeyAssembler for stateful ESC-sequence assembly in input.rs and a\npersistent cycle_start command in daemon.rs.  The Zsh plugin now sends\nraw key bytes over the Unix socket; the daemon interprets them and\nreplies with FRAME/DONE/NONE, replacing 6 widgets, 13 keybindings, and\n~250 lines of shell logic with a single catch-all _zacrs_cycle_handle\nwidget.\n\nKey changes:\n- input.rs: KeyAssembler + FeedResult, parse_single_byte extraction,\n  stack-based CSI reconstruction\n- daemon.rs: handle_cycle loop, setup_session/apply_navigation helpers\n- shell: daemon-driven cycle mode, deferred popup draw after keymap\n  switch to prevent ZSH redraw from erasing the initial frame\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(cycle): add Rust-owned subprocess fallback for cycle sessions\n\n* fix(input): make escape cancel cycle immediately via raw sequence forwarding\n\n* refactor(shell): simplify tab popup session\n\n* refactor(daemon): remove cycle session plumbing\n\n* fix(shell): retrigger popup after space completion\n\n* fix(shell): restore popup completion actions\n\n* fix(shell): clear popup before executing completion\n\n* fix(shell): refresh prompt before accepting line\n\n* fix(shell): restore popup session chaining and passthrough\n\n* fix(input): passthrough unhandled subprocess keys\n\n* fix(input): leave ctrl-j to zsh\n\n* fix(daemon): honor terminfo shift-tab sequences\n\n* fix(daemon): use is_multiple_of for clippy 1.94 lint\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(input): preserve terminal key sequences\n\n* fix(shell): pass terminfo shift-tab to complete fallback\n\n* fix(daemon): preserve utf-8 key input in popup sessions\n\n* fix(input): parse tty bytes with termwiz\n\n* test(daemon): cover control-key passthrough\n\n* test(input): document passthrough event policy\n\n* test(input): cover passthrough special keys\n\n* fix(test): satisfy clippy utf-8 length check\n\n* bench(daemon): isolate benchmark daemon\n\n* fix(bench): satisfy clippy for daemon helper\n\n* fix(input): relax split escape timeout\n\n* fix(daemon): drain oversized key payloads\n\n* fix(input): preserve long escape passthrough\n\n* fix(input): widen timing margin in long escape passthrough test\n\nReduce sender sleep from ESC_SEQUENCE_TIMEOUT/4 (12.5ms) to 2ms so\nthe data reliably arrives within the 50ms poll timeout on loaded\nmacOS CI runners.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(protocol): deduplicate decode_hex_bytes\n\nMove the shared hex-to-bytes decoder into protocol.rs so both main.rs\nand daemon.rs reference a single implementation.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): align escape sequence timeout with Rust side\n\nIncrease sysread timeout in _zacrs_read_key_input from 20ms to 50ms to\nmatch ESC_SEQUENCE_TIMEOUT in input.rs, reducing the risk of split\nescape sequences being misclassified on slower schedulers.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(daemon): move doc comment to correct function\n\nThe `cap_viewport_and_scroll` doc comment was incorrectly attached to\n`apply_navigation` after extraction.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(input): unify subprocess input path with daemon protocol\n\nThe subprocess `complete` command no longer opens /dev/tty directly.\nInstead, the shell spawns it as a coproc and communicates via\nstdin/stdout using the same KEY/FRAME/DONE text protocol as the daemon.\nThis eliminates ESC sequence timeout logic from Rust entirely — the\nshell handles key assembly — and removes the flaky CI tests that\ndepended on thread-sleep timing margins.\n\n- Add `--cols`/`--rows` to `complete` CLI (subprocess can't query\n  terminal size over pipes)\n- Add `daemon::run_stdio_complete()` public wrapper that reuses\n  `handle_complete<R: BufRead, W: Write>` over stdin/stdout\n- Rewrite `run_complete()` from ~130 lines to ~20 lines\n- Delete `TtyInputReader`, `read_key_bytes`, `poll_reader`,\n  `ESC_SEQUENCE_TIMEOUT`, `ReadOutcome`, `TtyGuard`, `AppResult`\n- Delete 4 timing-sensitive `read_key_bytes_*` tests\n- Extract `_zacrs_popup_session_loop` shared by daemon and coproc paths\n- Rewrite `_zacrs_invoke` from one-shot pipe to coproc protocol\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(main): remove redundant Config::load on complete path\n\nConfig was loaded unconditionally at the top of main() but only used\nby the Render subcommand. The Complete path loads its own config inside\nrun_stdio_complete, resulting in a double disk read + TOML parse.\nMove the load into the Render branch where it is actually needed.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* perf(shell): eliminate subprocess spawning per keystroke in popup loop (#44)\n\nReplace od|tr process spawning (2-4 per keystroke) with zsh builtins:\n- Byte counting: LC_ALL=C ${#var} instead of od -tx1 | tr\n- UTF-8 lead byte detection: printf -v (builtin) instead of od\n- UTF-8 accumulation: sysread call counter instead of re-encoding\n\nRemove _zacrs_input_nbytes and _zacrs_utf8_sequence_len (no longer needed).\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>\n\n* refactor(daemon): remove read_tsv thin wrapper\n\nMake read_tsv_payload pub directly instead of wrapping it\nwith an identical single-line function.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(input): clarify Action::None vs Option<Action> semantics\n\nparse_raw_bytes returns Action::None (ignored by render path).\nparse_tty_bytes_with_shift_tab returns Option, where None means\npassthrough to shell (DONE 3).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* Revert [profile.bench] from 36e5209\n\nMoved to a standalone PR (#45).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(shell): don't corrupt buffer on passthrough key in popup session (#46)\n\nWhen an unrecognized key (e.g. right arrow) was pressed in the popup,\nthe daemon returned DONE 3 with filter_text.  Two problems in\n_zacrs_apply_result caused breakage:\n\n1. LBUFFER was overwritten with filter_text (which includes the\n   auto-computed common prefix), clashing with stale autosuggestion\n   state and producing duplicated text like \"cargogo install --path .\".\n\n2. `unset POSTDISPLAY` cleared the zsh-autosuggestions ghost text\n   before the re-injected key could accept it.\n\nFix: for code 3 (passthrough), leave both LBUFFER and POSTDISPLAY\nunchanged so the re-injected key operates on the original shell state\nand autosuggestion acceptance works correctly.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-30T18:46:03+09:00",
          "tree_id": "bd1e6d7c67e28b9f02f83397c68863bedace5d5f",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/41c9d587b54067a96e1b08b5a16bbb8248099ab4"
        },
        "date": 1774864444081,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7323,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 74723,
            "range": "± 1444",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 904422,
            "range": "± 6337",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 190453,
            "range": "± 6691",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100085,
            "range": "± 2570",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 75521,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 19928,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 18836,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 11992,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 309940,
            "range": "± 3824",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 303385,
            "range": "± 6121",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 264892,
            "range": "± 5063",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 293849,
            "range": "± 3459",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 26344,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 301187,
            "range": "± 1038",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 3302139,
            "range": "± 17746",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 151692,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 113080,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 356832,
            "range": "± 14847",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 736,
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
            "value": 113,
            "range": "± 2",
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
            "value": 96,
            "range": "± 1",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/1field",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/2fields",
            "value": 54,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/3fields",
            "value": 69,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 856,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7604,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 754,
            "range": "± 5",
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
          "id": "94aac727fde2dc0556e9667ccdf287b37505d0a1",
          "message": "bench(daemon): add config hot-reload benchmarks (#69)\n\n* bench(daemon): add config hot-reload benchmarks\n\nAdd two benchmarks for the per-connection config reload path:\n- config_reload_mtime_check: steady-state stat() cost when config unchanged\n- config_reload_full: Config::load() + theme() + key_bindings() when config changed\n\nBoth use a temp config file for deterministic measurement.\n\nCloses #59\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* style(bench): use raw string for TOML constant and fix SAFETY comment\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-31T01:18:01+09:00",
          "tree_id": "3f46af52ca18cb905326456a14503aab21dd28b8",
          "url": "https://github.com/oyoshot/zsh-autocomplete-rs-proto/commit/94aac727fde2dc0556e9667ccdf287b37505d0a1"
        },
        "date": 1774887984073,
        "tool": "cargo",
        "benches": [
          {
            "name": "filter_scaling/100",
            "value": 7307,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/1000",
            "value": 75060,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "filter_scaling/10000",
            "value": 907440,
            "range": "± 37749",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/empty",
            "value": 189255,
            "range": "± 2444",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/1char",
            "value": 100991,
            "range": "± 486",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/3char",
            "value": 75646,
            "range": "± 523",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/exact",
            "value": 20256,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/no_match",
            "value": 19064,
            "range": "± 107",
            "unit": "ns/iter"
          },
          {
            "name": "filter_query_variants/long",
            "value": 12181,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/3char",
            "value": 386010,
            "range": "± 9287",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/normalized_exact",
            "value": 380039,
            "range": "± 11474",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/long_normalized",
            "value": 340575,
            "range": "± 11449",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_query_variants/no_match",
            "value": 367390,
            "range": "± 9605",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/100",
            "value": 35653,
            "range": "± 2114",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/1000",
            "value": 406787,
            "range": "± 20662",
            "unit": "ns/iter"
          },
          {
            "name": "filter_unicode_scaling/normalized_primary/10000",
            "value": 4453509,
            "range": "± 253211",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/full_rescan_git",
            "value": 153261,
            "range": "± 1173",
            "unit": "ns/iter"
          },
          {
            "name": "filter_sequence/incremental_git",
            "value": 113939,
            "range": "± 992",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/full_rescan_roundtrip_git",
            "value": 361452,
            "range": "± 1787",
            "unit": "ns/iter"
          },
          {
            "name": "app_backspace_sequence/app_cache_roundtrip_git",
            "value": 730,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_no_trunc",
            "value": 39,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "truncate_to_width/ascii_trunc",
            "value": 112,
            "range": "± 0",
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
            "value": 96,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse_line/long_desc",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/10",
            "value": 136,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/100",
            "value": 857,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/with_prefix/1000",
            "value": 7584,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "compute_common_prefix/no_prefix/1000",
            "value": 757,
            "range": "± 23",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}