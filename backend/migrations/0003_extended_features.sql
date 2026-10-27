-- Migration 0003: Extended Features (Category Tree, Customers, Shop Identity, Hero Config, Order Enhancements)

-- 1. Categories Hierarchy Table
CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id UUID REFERENCES categories(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_slug ON categories(slug);

-- Seed initial hierarchical categories - only if categories table is empty
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM categories) THEN
        INSERT INTO categories (id, parent_id, name, slug, description, display_order)
        VALUES
        ('30000000-0000-0000-0000-000000000001', NULL, 'Hardware', 'hardware', 'Precision-crafted mechanical and computing peripherals', 1),
        ('30000000-0000-0000-0000-000000000002', NULL, 'Apparel', 'apparel', 'High quality organic cotton developer apparel', 2),
        ('30000000-0000-0000-0000-000000000003', NULL, 'Software & Books', 'software-books', 'Systems architecture manuals, eBooks and crates', 3),
        ('30000000-0000-0000-0000-000000000004', NULL, 'Accessories', 'accessories', 'Desk mats, aviator cables, keycap pullers and tools', 4)
        ON CONFLICT (slug) DO NOTHING;

        -- Seed subcategories under Hardware
        INSERT INTO categories (id, parent_id, name, slug, description, display_order)
        VALUES
        ('30000000-0000-0000-0000-000000000011', '30000000-0000-0000-0000-000000000001', 'Keyboards', 'keyboards', 'Custom mechanical keyboards and kits', 1),
        ('30000000-0000-0000-0000-000000000012', '30000000-0000-0000-0000-000000000001', 'Switches & Lube', 'switches-lube', 'Mechanical switch packs and tuning lubricants', 2),
        ('30000000-0000-0000-0000-000000000013', '30000000-0000-0000-0000-000000000001', 'Keycaps', 'keycaps', 'PBT dye-sub and double-shot keycap sets', 3)
        ON CONFLICT (slug) DO NOTHING;

        -- Seed subcategories under Apparel
        INSERT INTO categories (id, parent_id, name, slug, description, display_order)
        VALUES
        ('30000000-0000-0000-0000-000000000021', '30000000-0000-0000-0000-000000000002', 'Hoodies', 'hoodies', 'Heavyweight French terry hoodies', 1),
        ('30000000-0000-0000-0000-000000000022', '30000000-0000-0000-0000-000000000002', 'T-Shirts', 't-shirts', 'Organic cotton developer tees', 2)
        ON CONFLICT (slug) DO NOTHING;

        -- Seed subcategories under Software & Books
        INSERT INTO categories (id, parent_id, name, slug, description, display_order)
        VALUES
        ('30000000-0000-0000-0000-000000000031', '30000000-0000-0000-0000-000000000003', 'eBooks & Guides', 'ebooks-guides', 'Digital guides in PDF, ePub and Mobi', 1),
        ('30000000-0000-0000-0000-000000000032', '30000000-0000-0000-0000-000000000003', 'Software Licenses', 'software-licenses', 'Binary licenses and developer tools', 2)
        ON CONFLICT (slug) DO NOTHING;
    END IF;
END $$;

-- 2. Store Settings Extensions (Logo, Phone, Hero Configuration)
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS logo_url TEXT NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS phone VARCHAR(50) NOT NULL DEFAULT '+49 (0) 30 123456-78';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS hero_config JSONB NOT NULL DEFAULT '{
  "layout": "split",
  "carousel_items": [
    {
      "id": "c1",
      "product_id": "a1000000-0000-0000-0000-000000000001",
      "title": "RustCraft Pro 75% Mechanical Keyboard",
      "subtitle": "Hot-swappable CNC anodized aluminum chassis with QMK/VIA firmware",
      "image_url": "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=1400&q=80",
      "link_url": "/products/rust-mechanical-keyboard",
      "button_text": "Discover Precision"
    },
    {
      "id": "c2",
      "product_id": "a1000000-0000-0000-0000-000000000002",
      "title": "Rustacean Heavyweight Hoodie",
      "subtitle": "480 GSM French Terry organic cotton with embroidered Ferris insignia",
      "image_url": "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=1400&q=80",
      "link_url": "/products/rustacean-heavyweight-hoodie",
      "button_text": "Gear Up"
    }
  ],
  "featured_buttons": [
    {
      "id": "b1",
      "product_id": "a1000000-0000-0000-0000-000000000001",
      "title": "Mechanical Keyboards",
      "subtitle": "From 189.00 €",
      "image_url": "https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=600&q=80",
      "link_url": "/products/rust-mechanical-keyboard",
      "bg_color": "#ea580c"
    },
    {
      "id": "b2",
      "product_id": "a1000000-0000-0000-0000-000000000002",
      "title": "Heavyweight Hoodies",
      "subtitle": "From 79.00 €",
      "image_url": "https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=600&q=80",
      "link_url": "/products/rustacean-heavyweight-hoodie",
      "bg_color": "#0284c7"
    },
    {
      "id": "b3",
      "product_id": "a1000000-0000-0000-0000-000000000003",
      "title": "Systems Architecture Guide",
      "subtitle": "From 29.00 €",
      "image_url": "https://images.unsplash.com/photo-1532012164546-f432f2e37b73?auto=format&fit=crop&w=600&q=80",
      "link_url": "/products/zero-cost-abstractions-guide",
      "bg_color": "#16a34a"
    },
    {
      "id": "b4",
      "product_id": "a1000000-0000-0000-0000-000000000004",
      "title": "Aviator Coiled USB Cable",
      "subtitle": "From 34.00 €",
      "image_url": "https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=600&q=80",
      "link_url": "/products/rust-mechanical-keyboard",
      "bg_color": "#7c3aed"
    }
  ]
}';

-- 3. Customers Table
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL DEFAULT '',
    last_name VARCHAR(100) NOT NULL DEFAULT '',
    display_name VARCHAR(100) NOT NULL DEFAULT '',
    preferred_currency VARCHAR(10) NOT NULL DEFAULT 'EUR',
    phone VARCHAR(50) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_customers_email ON customers(email);

-- Ensure orders have customer_phone
ALTER TABLE orders ADD COLUMN IF NOT EXISTS customer_phone VARCHAR(50) NOT NULL DEFAULT '';
