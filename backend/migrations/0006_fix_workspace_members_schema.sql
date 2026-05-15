-- Fix workspace_members schema mismatch between 0001 and 0004.
-- 0001 creates the table with (workspace_id, user_id, role, joined_at)
-- 0004 tries CREATE TABLE IF NOT EXISTS with (id, invited_by, created_at)
-- but never runs because the table already exists.

-- Add missing 'id' column
ALTER TABLE workspace_members ADD COLUMN IF NOT EXISTS id UUID DEFAULT gen_random_uuid();

-- Add missing 'invited_by' column
ALTER TABLE workspace_members ADD COLUMN IF NOT EXISTS invited_by UUID REFERENCES users(id);

-- Add 'created_at' column (0001 used 'joined_at' instead)
ALTER TABLE workspace_members ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT now();

-- Copy data from joined_at to created_at if joined_at exists
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'workspace_members' AND column_name = 'joined_at'
    ) THEN
        UPDATE workspace_members SET created_at = joined_at WHERE created_at = now();
    END IF;
END $$;

-- Fix role CHECK constraint: 0001 uses ('admin','editor','viewer')
-- but code and 0004 use ('owner','editor','viewer')
ALTER TABLE workspace_members DROP CONSTRAINT IF EXISTS workspace_members_role_check;
ALTER TABLE workspace_members ADD CONSTRAINT workspace_members_role_check
    CHECK (role IN ('owner', 'admin', 'editor', 'viewer'));
