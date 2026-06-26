import { github, project, service } from "railway/iac";

const scoreboard = service("fgc-scoreboard", {
  source: github("wukrit/fgc-scoreboard"),
  build: {
    builder: "DOCKERFILE",
    dockerfilePath: "Dockerfile",
  },
  // Dockerfile CMD is /app/fgc-server; keep start in sync for Railway overrides.
  start: "/app/fgc-server",
  healthcheck: "/health",
  env: {
    FGC_RATE_LIMIT: "60",
    // FGC_AUTH_TOKEN: set after apply via `railway variables set` (see deploy/railway.md).
    // Do not add FGC_AUTH_TOKEN here — Railway IaC would delete it on apply if omitted.
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
