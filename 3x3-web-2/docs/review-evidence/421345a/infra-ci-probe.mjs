import fs from "node:fs";
import path from "node:path";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const dir = path.dirname(fileURLToPath(import.meta.url));
const workflow = fs.readFileSync(
  "../.github/workflows/deploy-pages.yml",
  "utf8",
);
const ci = fs.readFileSync("../.github/workflows/3x3-web-2.yml", "utf8");
const config = fs.readFileSync("playwright.config.ts", "utf8");
const condition = workflow.match(/^\s+if:\s*(github\.event_name[^\n]+)/m)?.[1];
assert.equal(condition, "github.event_name == 'push'");
const eventResults = Object.fromEntries(
  ["push", "pull_request", "workflow_dispatch"].map((event) => [
    event,
    event === "push",
  ]),
);
assert.equal(eventResults.workflow_dispatch, false);
assert.match(config, /reporter:\s*"list"/);
assert.match(ci, /path:\s*3x3-web-2\/playwright-report\//);
const scratch = path.relative(
  process.cwd(),
  path.join(dir, "infra-ci-scratch"),
);
fs.mkdirSync(scratch, { recursive: true });
let runner;
try {
  runner = spawnSync(
    process.execPath,
    [
      "node_modules/@playwright/test/cli.js",
      "test",
      "--config",
      path.relative(
        process.cwd(),
        path.join(dir, "infra-ci-fixture.config.mjs"),
      ),
    ],
    {
      cwd: process.cwd(),
      env: { ...process.env, TMPDIR: path.resolve(scratch) },
      encoding: "utf8",
      timeout: 20000,
    },
  );
  assert.equal(runner.status, 1, runner.stderr);
  const result = {
    deployCondition: condition,
    deployRunsByEvent: eventResults,
    reporterFixtureExit: runner.status,
    configuredReporter: "list",
    htmlReportExists: fs.existsSync(path.join(dir, "playwright-report")),
    failureArtifactsExist: fs.existsSync(
      path.join(dir, "infra-ci-test-results"),
    ),
    conclusion:
      "The CI reporter setting creates no HTML report for the configured upload path.",
  };
  assert.equal(result.htmlReportExists, false);
  fs.writeFileSync(
    path.relative(process.cwd(), path.join(dir, "infra-ci-results.json")),
    JSON.stringify(result, null, 2) + "\n",
  );
  fs.writeFileSync(
    path.relative(process.cwd(), path.join(dir, "infra-ci-fixture-output.txt")),
    runner.stdout + runner.stderr,
  );
  console.log(JSON.stringify(result, null, 2));
} finally {
  fs.rmSync(scratch, { recursive: true, force: true });
  fs.rmSync(
    path.relative(process.cwd(), path.join(dir, "infra-ci-test-results")),
    { recursive: true, force: true },
  );
}
