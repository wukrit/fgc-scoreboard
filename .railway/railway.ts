import { defineRailway, preserve, project, service } from "railway/iac";

export default defineRailway(() => {
  const scoreboard = service("fgc-scoreboard", {
    start: "python3 server.py",
    healthcheck: "/health",
    env: {
      FGC_RATE_LIMIT: "60",
      // Set once via dashboard/CLI; preserve() prevents apply from wiping it
      FGC_AUTH_TOKEN: preserve(),
    },
    // Optional: add custom domain when ready, then railway config pull to sync
    // domains: ["scoreboard.example.com"],
  });

  return project("fgc-scoreboard", {
    resources: [scoreboard],
  });
});
