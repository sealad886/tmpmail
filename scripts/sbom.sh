#!/bin/sh
set -eu

output=${1:-dist/tmpmail.cdx.json}
mkdir -p "$(dirname "$output")"
metadata=$(cargo metadata --locked --format-version 1)
printf '%s\n' "$metadata" | jq '
  . as $metadata |
  (.packages[] | select(.id == $metadata.resolve.root)) as $application |
  {
  bomFormat: "CycloneDX",
  specVersion: "1.5",
  version: 1,
  metadata: {component: {type: "application", name: $application.name, version: $application.version}},
  components: [.packages[] | {
    type: "library",
    name: .name,
    version: .version,
    licenses: (if .license then [{expression: .license}] else [] end),
    purl: ("pkg:cargo/" + .name + "@" + .version)
  }]
}' > "$output"
