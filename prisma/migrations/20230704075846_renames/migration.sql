/*
  Warnings:

  - You are about to drop the column `remind_every` on the `User` table. All the data in the column will be lost.
  - You are about to drop the column `reminded_at` on the `User` table. All the data in the column will be lost.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_User" (
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "chatId" BIGINT NOT NULL PRIMARY KEY,
    "remindEvery" INTEGER NOT NULL DEFAULT 1800,
    "nextRemindAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO "new_User" ("chatId", "createdAt", "updatedAt") SELECT "chatId", "createdAt", "updatedAt" FROM "User";
DROP TABLE "User";
ALTER TABLE "new_User" RENAME TO "User";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
