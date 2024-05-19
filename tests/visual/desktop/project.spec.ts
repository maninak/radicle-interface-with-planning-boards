import {
  test,
  expect,
  cobUrl,
  sourceBrowsingUrl,
  aliceRemote,
  markdownUrl,
  sourceBrowsingRid,
} from "@tests/support/fixtures.js";
import sinon from "sinon";

test("source page", async ({ page }) => {
  await page.goto(sourceBrowsingUrl, { waitUntil: "networkidle" });
  await expect(page).toHaveScreenshot();
});

test("history page", async ({ page }) => {
  await page.addInitScript(() => {
    sinon.useFakeTimers({
      now: new Date("November 24 2022 12:00:00").valueOf(),
      shouldClearNativeTimers: true,
      shouldAdvanceTime: false,
    });
  });

  await page.goto(
    `${sourceBrowsingUrl}/remotes/${aliceRemote.substring(8)}/history`,
    {
      waitUntil: "networkidle",
    },
  );

  await expect(page).toHaveScreenshot({ fullPage: true });
});

test("commit page", async ({ page }) => {
  await page.addInitScript(() => {
    sinon.useFakeTimers({
      now: new Date("November 24 2022 12:00:00").valueOf(),
      shouldClearNativeTimers: true,
      shouldAdvanceTime: false,
    });
  });

  await page.goto(
    `${sourceBrowsingUrl}/remotes/${aliceRemote.substring(
      8,
    )}/commits/1aded56c3ad55299df9f06c326af50b802a05949`,
  );
  await expect(page.getByText("subconscious.txt added")).toBeVisible();
  await expect(page).toHaveScreenshot({ fullPage: true });
});

test("diff selection", async ({ page }) => {
  await page.addInitScript(() => {
    sinon.useFakeTimers({
      now: new Date("November 24 2022 12:00:00").valueOf(),
      shouldClearNativeTimers: true,
      shouldAdvanceTime: false,
    });
  });

  await page.goto(`${cobUrl}/patches`);
  await page
    .getByRole("link", { name: "Taking another stab at the README" })
    .click();
  await page.getByRole("link", { name: "Changes" }).click();
  await page
    .getByRole("row", { name: "- # Cobs Repo" })
    .locator("div")
    .first()
    .click();
  await page.keyboard.down("Shift");
  await page
    .getByRole("row", { name: "+ ## Better?" })
    .locator("div")
    .first()
    .click();
  await page.keyboard.up("Shift");
  await expect(page).toHaveScreenshot({ fullPage: true });
});

test("project load error", async ({ page }) => {
  await page.goto(
    `${sourceBrowsingUrl}/remotes/zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz`,
    { waitUntil: "networkidle" },
  );
  await expect(page).toHaveScreenshot();
});

test("project not found", async ({ page }) => {
  await page.goto(`/nodes/127.0.0.1/rad:z4Vzzzzzzzzzzzzzzzzzzzzzzzzzz`, {
    waitUntil: "networkidle",
  });
  await expect(page).toHaveScreenshot();
});

test("response parse error", async ({ page }) => {
  await page.route(
    ({ pathname }) => pathname === `/api/v1/projects/${sourceBrowsingRid}`,
    route => {
      return route.fulfill({
        json: [{ name: 1337 }],
      });
    },
  );
  await page.goto(sourceBrowsingUrl, { waitUntil: "networkidle" });
  await expect(page).toHaveScreenshot();
});

test("response error", async ({ page }) => {
  await page.route(
    ({ pathname }) => pathname === `/api/v1/projects/${sourceBrowsingRid}`,
    route => {
      return route.fulfill({
        status: 500,
      });
    },
  );
  await page.goto(sourceBrowsingUrl, { waitUntil: "networkidle" });
  await expect(page).toHaveScreenshot();
});

test("readme not found", async ({ page }) => {
  await page.goto(`${markdownUrl}/tree`, {
    waitUntil: "networkidle",
  });
  await expect(page).toHaveScreenshot();
});

test("file not found", async ({ page }) => {
  await page.goto(`${sourceBrowsingUrl}/tree/this.file.does.not.exist`, {
    waitUntil: "networkidle",
  });
  await expect(page).toHaveScreenshot();
});

test("commit not found", async ({ page }) => {
  await page.goto(
    `${sourceBrowsingUrl}/commits/0000000000000000000000000000000000000000`,
    { waitUntil: "networkidle" },
  );
  await expect(page).toHaveScreenshot();
});

test("issue not found", async ({ page }) => {
  await page.goto(
    `${sourceBrowsingUrl}/issues/0000000000000000000000000000000000000000`,
    { waitUntil: "networkidle" },
  );
  await expect(page).toHaveScreenshot();
});

test("patch not found", async ({ page }) => {
  await page.goto(
    `${sourceBrowsingUrl}/patches/0000000000000000000000000000000000000000`,
    { waitUntil: "networkidle" },
  );
  await expect(page).toHaveScreenshot();
});
