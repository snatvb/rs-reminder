/*
  Warnings:

  - You are about to drop the column `usersChatId` on the `Word` table. All the data in the column will be lost.
  - Added the required column `userChatId` to the `Word` table without a default value. This is not possible if the table is not empty.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_Word" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "chatId" BIGINT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "word" TEXT NOT NULL,
    "translate" TEXT NOT NULL,
    "rememberLevel" INTEGER NOT NULL DEFAULT 0,
    "nextRemindAt" DATETIME NOT NULL,
    "remindedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "userChatId" BIGINT NOT NULL,
    CONSTRAINT "Word_userChatId_fkey" FOREIGN KEY ("userChatId") REFERENCES "User" ("chatId") ON DELETE RESTRICT ON UPDATE CASCADE
);
INSERT INTO "new_Word" ("chatId", "createdAt", "id", "nextRemindAt", "rememberLevel", "remindedAt", "translate", "updatedAt", "word") SELECT "chatId", "createdAt", "id", "nextRemindAt", "rememberLevel", "remindedAt", "translate", "updatedAt", "word" FROM "Word";
DROP TABLE "Word";
ALTER TABLE "new_Word" RENAME TO "Word";
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
