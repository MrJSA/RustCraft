-- Migration 0005: Cookie Consent, Product Multi-Image Gallery, SEO Subtitles, Long Markdown Descriptions, Category Visuals, and Menu Ordering

-- 1. Store Settings: EU Cookie Consent Banner
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_banner_enabled BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_banner_title VARCHAR(255) NOT NULL DEFAULT 'We respect your privacy';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_banner_description TEXT NOT NULL DEFAULT 'We use cookies and similar technologies to enhance your browsing experience, analyze site traffic, and personalize content in accordance with EU GDPR.';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_banner_policy_url VARCHAR(255) NOT NULL DEFAULT '/policies/cookie-policy';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_accept_label VARCHAR(100) NOT NULL DEFAULT 'Accept All';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_deny_label VARCHAR(100) NOT NULL DEFAULT 'Decline Optional';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS cookie_preferences_label VARCHAR(100) NOT NULL DEFAULT 'Preferences';

-- 2. Products Extensions: Subtitle, Custom Variant Selector Label, Short & Long Markdown Descriptions, Gallery Images
ALTER TABLE products ADD COLUMN IF NOT EXISTS subtitle VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE products ADD COLUMN IF NOT EXISTS variant_selector_label VARCHAR(100) NOT NULL DEFAULT 'Choose Variant / Model:';
ALTER TABLE products ADD COLUMN IF NOT EXISTS short_description TEXT NOT NULL DEFAULT '';
ALTER TABLE products ADD COLUMN IF NOT EXISTS long_description TEXT NOT NULL DEFAULT '';
ALTER TABLE products ADD COLUMN IF NOT EXISTS images JSONB NOT NULL DEFAULT '[]';

-- 3. Product Variants Extensions: Variant Gallery Images
ALTER TABLE product_variants ADD COLUMN IF NOT EXISTS images JSONB NOT NULL DEFAULT '[]';

-- 4. Categories Extensions: Category Image
ALTER TABLE categories ADD COLUMN IF NOT EXISTS image_url TEXT NOT NULL DEFAULT '';

-- 5. Data Backfill
-- Set short_description and long_description from description if empty
UPDATE products 
SET short_description = description 
WHERE (short_description IS NULL OR short_description = '') AND description IS NOT NULL;

UPDATE products 
SET long_description = description 
WHERE (long_description IS NULL OR long_description = '') AND description IS NOT NULL;

-- Backfill images array with primary image_url if images is empty
UPDATE products 
SET images = jsonb_build_array(image_url) 
WHERE (images IS NULL OR images = '[]'::jsonb) AND image_url IS NOT NULL AND image_url != '';

UPDATE product_variants 
SET images = jsonb_build_array(image_url) 
WHERE (images IS NULL OR images = '[]'::jsonb) AND image_url IS NOT NULL AND image_url != '';

-- Seed category images if empty
UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'hardware' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'apparel' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1532012164546-f432f2e3edd4?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'software-books' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1616440347437-b1c73416efc2?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'accessories' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'keyboards' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1595225476474-87563907a212?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'switches-lube' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1618384887929-16ec33fab9ef?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'keycaps' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1556905055-8f358a7a47b2?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'hoodies' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1521572267360-ee0c2909d518?auto=format&fit=crop&w=600&q=80'
WHERE slug = 't-shirts' AND (image_url IS NULL OR image_url = '');

UPDATE categories 
SET image_url = 'https://images.unsplash.com/photo-1532012164546-f432f2e3edd4?auto=format&fit=crop&w=600&q=80'
WHERE slug = 'ebooks-guides' AND (image_url IS NULL OR image_url = '');

-- Update Mechanical Keyboard with multiple images and subtitles
UPDATE products
SET 
  subtitle = 'Hot-Swappable 75% CNC Aluminum Mechanical Keyboard with QMK/VIA',
  variant_selector_label = 'Choose Switch Type & Color:',
  short_description = 'High-end hot-swappable mechanical keyboard engineered with CNC aluminum chassis, customized Rust orange keycaps, and ultra-low latency response.',
  long_description = '### Engineering Excellence for Systems Developers

Crafted from solid 6063 aerospace-grade aluminum, the **RustCraft Pro 75%** is machined via precision 5-axis CNC and bead-blasted to an ultra-fine velvet matte texture.

#### Key Highlights & Specifications:
- **PCB Architecture**: South-facing per-key RGB LEDs, hot-swappable Kailh sockets rated for 10,000 swap cycles, and hardware debouncing.
- **Acoustic Tuning**: Multi-layer Poron gasket mounting paired with IXPE switch pads and molded silicone bottom case dampening.
- **Open-Source Firmware**: Full support for QMK, VIA, and VIAL with instant zero-reboot key remapping and multi-layer macros.
- **Connectivity**: Gold-plated USB-C port with electrostatic discharge (ESD) protection and custom paracord aviator cable included.

Each board is individually numbered and factory-inspected for key switch alignment, stabilizer wobble, and structural rigidity.',
  images = jsonb_build_array(
    'https://images.unsplash.com/photo-1587829741301-dc798b83add3?auto=format&fit=crop&w=1200&q=80',
    'https://images.unsplash.com/photo-1595225476474-87563907a212?auto=format&fit=crop&w=1200&q=80',
    'https://images.unsplash.com/photo-1618384887929-16ec33fab9ef?auto=format&fit=crop&w=1200&q=80',
    'https://images.unsplash.com/photo-1544716278-ca5e3f4abd8c?auto=format&fit=crop&w=1200&q=80'
  )
WHERE slug = 'rust-mechanical-keyboard';
