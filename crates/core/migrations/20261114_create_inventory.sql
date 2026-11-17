-- Inventory / LIMS module (Fase I+D)
-- Created: 2026-11-14

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    item_type TEXT NOT NULL DEFAULT 'reagent',
    location TEXT,
    quantity REAL NOT NULL DEFAULT 0,
    unit TEXT,
    lot_number TEXT,
    expiry_date TIMESTAMPTZ,
    owner_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inventory_items_type ON inventory_items(item_type);
CREATE INDEX IF NOT EXISTS idx_inventory_items_owner ON inventory_items(owner_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_expiry ON inventory_items(expiry_date);
CREATE INDEX IF NOT EXISTS idx_inventory_items_name ON inventory_items(name);
