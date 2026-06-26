import { github, preserve, project, service } from "railway/iac";

const scoreboard = service("fgc-scoreboard", {
  source: github("wukrit/fgc-scoreboard"),
  build: {
    builder: "DOCKERFILE",
    dockerfilePath: "Dockerfile",
  },
  // Dockerfile CMD is /app/fgc-server --no-tunnel; keep start in sync for Railway overrides.
  start: "/app/fgc-server --no-tunnel",
  healthcheck: "/health",
  env: {
    FGC_RATE_LIMIT: "60",
    // Keep existing token on config apply; set initially via CLI (see deploy/railway.md).
    FGC_AUTH_TOKEN: preserve(),
  },
  deploy: {
    limitOverride: {
      containers: {
        cpu: 2,
        memoryBytes: 2_000_000_000,
      },
    },
  },
  domains: ["satorha-is-a-bum.kritgpt.com"],
});

// module.exports avoids a tsx ESM default-export interop bug (nested { default })
// that makes `railway config plan` see an empty graph.
module.exports = project("fgc-scoreboard", {
  resources: [scoreboard],
});
