import { github, project, service } from "railway/iac";

const scoreboard = service("fgc-scoreboard", {
  source: github("wukrit/fgc-scoreboard"),
  start: "python3 server.py",
  healthcheck: "/health",
  env: {
    FGC_RATE_LIMIT: "60",
    // FGC_AUTH_TOKEN: set after apply via `railway variables set` (see deploy/railway.md).
    // Do not use preserve() here — Railway rejects { type: "preserve" } on service create.
  },
  // Optional: add custom domain when ready, then railway config pull to sync
  // domains: ["scoreboard.example.com"],
});

// module.exports avoids a tsx ESM default-export interop bug (nested { default })
// that makes `railway config plan` see an empty graph.
module.exports = project("fgc-scoreboard", {
  resources: [scoreboard],
});
