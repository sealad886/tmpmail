#!/bin/sh
set -eu

output=${1:-dist/tmpmail.cdx.json}
mkdir -p "$(dirname "$output")"
cargo metadata --locked --format-version 1 | jq '{
  bomFormat: "CycloneDX",
  specVersion: "1.5",
  version: 1,
  metadata: {component: {type: "application", name: "tmpmail", version: "2.0.0"}},
  components: [.packages[] | {
    type: "library",
    name: .name,
    version: .version,
    licenses: (if .license then [{expression: .license}] else [] end),
    purl: ("pkg:cargo/" + .name + "@" + .version)
  }]
}' > "$output"
