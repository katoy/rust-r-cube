import { test, expect } from "@playwright/test";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import config from "../playwright.config";

const projectDir = fileURLToPath(new URL("../", import.meta.url));
const workflowDir = path.resolve(projectDir, "../.github/workflows");

test("Pages deploy permits only main push/manual events after the build gate", () => {
  const workflow = fs.readFileSync(
    path.join(workflowDir, "deploy-pages.yml"),
    "utf8",
  );
  const deploy = workflow.slice(workflow.indexOf("\n  deploy:"));
  expect(deploy).toMatch(/\n    needs: build\n/);
  const expression = deploy.match(/\n    if: (.+)\n/)?.[1];
  expect(expression).toBeTruthy();
  // Evaluate the trusted workflow's boolean expression, not a copy of its policy.
  const allowsDeploy = new Function(
    "github",
    `return (${expression});`,
  ) as (github: { ref: string; event_name: string }) => boolean;
  for (const event_name of [
    "push",
    "workflow_dispatch",
    "pull_request",
    "pull_request_target",
  ]) {
    for (const ref of [
      "refs/heads/main",
      "refs/heads/master",
      "refs/heads/feature/test",
      "refs/pull/42/merge",
    ]) {
      expect(allowsDeploy({ ref, event_name })).toBe(
        ref === "refs/heads/main" &&
          ["push", "workflow_dispatch"].includes(event_name),
      );
    }
  }
  expect(workflow).toMatch(/\n    needs: \[test-3x3-web-2\]\n/);
});

test("CI failure uploads use Playwright's actual evidence directory", () => {
  expect(config.reporter).toBe("list");
  expect(config.use?.screenshot).toBe("only-on-failure");
  expect(config.use?.trace).toBe("retain-on-failure");
  for (const name of ["3x3-web-2.yml", "deploy-pages.yml"]) {
    const workflow = fs.readFileSync(path.join(workflowDir, name), "utf8");
    const artifactStep = workflow.match(
      /- name: Upload Playwright failure evidence\n([\s\S]*?)(?=\n      - name:|\n  [a-z]|$)/,
    )?.[1];
    expect(artifactStep).toBeTruthy();
    expect(artifactStep).toContain("if: failure()");
    expect(artifactStep).toContain("uses: actions/upload-artifact@v4");
    expect(artifactStep).toContain(`path: 3x3-web-2/${config.outputDir}/`);
  }
});
