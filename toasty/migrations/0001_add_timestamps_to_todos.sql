ALTER TABLE "todos" ADD COLUMN "created_at" TEXT NOT NULL;
-- #[toasty::breakpoint]
ALTER TABLE "todos" ADD COLUMN "updated_at" TEXT NOT NULL;
