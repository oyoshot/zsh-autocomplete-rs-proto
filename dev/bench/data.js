window.BENCHMARK_DATA = {
  "lastUpdate": 1774010765207,
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
      }
    ]
  }
}