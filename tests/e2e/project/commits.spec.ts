import {
  aliceMainCommitCount,
  aliceMainCommitMessage,
  aliceMainHead,
  bobMainCommitCount,
  expect,
  gitOptions,
  shortBobHead,
  sourceBrowsingUrl,
  test,
} from "@tests/support/fixtures.js";
import { createProject } from "@tests/support/project";
import sinon from "sinon";

test("peer and branch switching", async ({ page }) => {
  await page.goto(sourceBrowsingUrl);
  await page
    .getByRole("link", { name: `Commits ${aliceMainCommitCount}` })
    .click();

  // Alice's peer.
  {
    await page.getByTitle("Change peer").click();
    await page
      .getByRole("link", {
        name: "alice delegate",
      })
      .click();

    await expect(page.getByTitle("Change peer")).toHaveText("alice Delegate");

    await expect(page.getByText("Thursday, November 17, 2022")).toBeVisible();
    await expect(page.locator(".list .teaser")).toHaveCount(
      aliceMainCommitCount,
    );

    const latestCommit = page.locator(".teaser").first();
    await expect(latestCommit).toContainText(aliceMainCommitMessage);
    await expect(latestCommit).toContainText(aliceMainHead.substring(0, 7));

    const earliestCommit = page.locator(".teaser").last();
    await expect(earliestCommit).toContainText(
      "Initialize an empty git repository",
    );
    await expect(earliestCommit).toContainText("36d5bbe");

    await page.getByTitle("Change branch").click();
    await page.getByText("feature/branch").click();

    await expect(
      page.getByRole("button", { name: "feature/branch" }),
    ).toBeVisible();
    await expect(page.getByText("Thursday, November 17, 2022")).toBeVisible();
    await expect(page.locator(".list .teaser")).toHaveCount(bobMainCommitCount);

    await page.getByTitle("Change branch").click();
    await page.getByText("orphaned-branch").click();

    await expect(
      page.getByRole("button", { name: "orphaned-branch" }),
    ).toBeVisible();
    await expect(page.getByText("Thursday, November 17, 2022")).toBeVisible();
    await expect(page.locator(".list")).toHaveCount(1);
  }

  // Bob's peer.
  {
    await page.getByTitle("Change peer").click();
    await page.getByRole("link", { name: "bob" }).click();

    await expect(page.getByTitle("Change peer")).toContainText("bob");

    await expect(page.getByText("Wednesday, December 21, 2022")).toBeVisible();
    await expect(page.locator(".list").first().locator(".teaser")).toHaveCount(
      1,
    );

    await expect(page.getByText("Thursday, November 17, 2022")).toBeVisible();
    await expect(page.locator(".list").last().locator(".teaser")).toHaveCount(
      7,
    );

    const latestCommit = page.locator(".teaser").first();
    await expect(latestCommit).toContainText("Update readme");
    await expect(latestCommit).toContainText(shortBobHead);

    const earliestCommit = page.locator(".teaser").last();
    await expect(earliestCommit).toContainText(
      "Initialize an empty git repository",
    );
    await expect(earliestCommit).toContainText("36d5bbe");
  }
});

test("commit messages with double colon not converted into single colon", async ({
  page,
}) => {
  const commitMessage = "Verify that crate::DoubleColon::should_work()";
  const shortCommit = "7babd25";
  await page.goto(sourceBrowsingUrl);
  await page
    .getByRole("link", { name: `Commits ${aliceMainCommitCount}` })
    .click();

  await expect(
    page.getByRole("button", {
      name: `${shortCommit} ${commitMessage}`,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: commitMessage,
      exact: true,
    }),
  ).toBeVisible();

  await page
    .getByRole("link", {
      name: commitMessage,
      exact: true,
    })
    .click();
  await expect(page.getByText(commitMessage, { exact: true })).toBeVisible();
});

test("expand commit message", async ({ page }) => {
  await page.goto(sourceBrowsingUrl);
  await page
    .getByRole("link", { name: `Commits ${aliceMainCommitCount}` })
    .click();
  const commitToggle = page.getByRole("button", { name: "expand" }).first();

  await commitToggle.click();
  const expandedCommit = page.getByText(
    "This shouldn't replace double colons with simple colons",
  );

  await expect(expandedCommit).toBeVisible();
  await commitToggle.click();

  await expect(expandedCommit).toBeHidden();
});

test("relative timestamps", async ({ page }) => {
  await page.addInitScript(() => {
    sinon.useFakeTimers({
      now: new Date("December 21 2022 12:00:00").valueOf(),
      shouldClearNativeTimers: true,
      shouldAdvanceTime: false,
    });
  });

  await page.goto(sourceBrowsingUrl);
  await page
    .getByRole("link", { name: `Commits ${aliceMainCommitCount}` })
    .click();

  await page.getByTitle("Change peer").click();
  await page.getByRole("link", { name: "bob" }).click();
  await expect(page.getByTitle("Change peer")).toHaveText("bob");
  const latestCommit = page.locator(".teaser").first();
  await expect(latestCommit).toContainText(
    `Bob Belcher committed ${shortBobHead} now`,
  );
  await expect(latestCommit).toContainText(shortBobHead);
  const earliestCommit = page.locator(".teaser").last();
  await expect(earliestCommit).toContainText(
    "Alice Liddell committed 36d5bbe last month",
  );
});

test("pushing changes while viewing history", async ({ page, peerManager }) => {
  const alice = await peerManager.createPeer({
    name: "alice",
    gitOptions: gitOptions["alice"],
  });
  await alice.startNode();
  await alice.startHttpd();
  const { rid, projectFolder } = await createProject(alice, {
    name: "alice-project",
  });
  await page.goto(`${alice.uiUrl()}/${rid}`);
  await page.getByRole("link", { name: "Commits 1" }).click();
  await expect(page).toHaveURL(`${alice.uiUrl()}/${rid}/history`);

  await alice.git(["commit", "--allow-empty", "--message", "first change"], {
    cwd: projectFolder,
  });
  await alice.git(["push", "rad", "main"], {
    cwd: projectFolder,
  });
  await page.reload();
  await expect(page).toHaveURL(`${alice.uiUrl()}/${rid}/history`);
  await expect(page.getByRole("link", { name: "Commits 2" })).toBeVisible();

  await expect(page.getByTitle("Change branch")).toHaveText("main Canonical");
  const branchSelectorCommitButton = page.getByTitle("Current HEAD").first();
  await expect(branchSelectorCommitButton).toHaveText("516fa74 first change");

  await page
    .getByRole("banner")
    .getByRole("link", { name: "alice-project" })
    .click();
  await expect(page).toHaveURL(`${alice.uiUrl()}/${rid}`);
  await page.getByRole("link", { name: "Commits 2" }).click();

  await alice.git(
    [
      "commit",
      "--allow-empty",
      "--message",
      "after clicking the project title",
    ],
    {
      cwd: projectFolder,
    },
  );
  await alice.git(["push", "rad", "main"], {
    cwd: projectFolder,
  });
  await page.reload();
  await expect(page).toHaveURL(`${alice.uiUrl()}/${rid}/history`);
  await expect(page.getByRole("link", { name: "Commits 3" })).toHaveText(
    "Commits 3",
  );
  await expect(
    page.getByRole("button", { name: "main Canonical" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "bb9089a after clicking the" }),
  ).toBeVisible();
});
