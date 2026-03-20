window.BENCHMARK_DATA = {
  "lastUpdate": 1774018821373,
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
      }
    ]
  }
}