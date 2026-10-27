-- Migration 0002: Enhanced Features (Hierarchical Shipping, Product Parts BOM, Policy CMS, Dynamic Navigation, Customer Portal)

-- 1. Shipping Providers
CREATE TABLE IF NOT EXISTS shipping_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) NOT NULL UNIQUE,
    tracking_url_template TEXT NOT NULL DEFAULT 'https://www.dhl.com/track?id={tracking_number}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial shipping providers
INSERT INTO shipping_providers (id, name, code, tracking_url_template, is_active, sort_order)
VALUES 
('10000000-0000-0000-0000-000000000001', 'DHL Paket & Express', 'dhl', 'https://www.dhl.de/de/privatkunden/pakete-verfolgen.html?piececode={tracking_number}', true, 1),
('10000000-0000-0000-0000-000000000002', 'Hermes Einrichtungs Service', 'hermes', 'https://www.myhermes.de/empfangen/sendungsverfolgung/?tracking_id={tracking_number}', true, 2),
('10000000-0000-0000-0000-000000000003', 'UPS Worldwide Courier', 'ups', 'https://www.ups.com/track?tracknum={tracking_number}', true, 3)
ON CONFLICT (code) DO NOTHING;

-- Link shipping_zones to shipping_providers
ALTER TABLE shipping_zones ADD COLUMN IF NOT EXISTS provider_id UUID REFERENCES shipping_providers(id) ON DELETE CASCADE;

-- Update existing zones to associate with DHL by default
UPDATE shipping_zones SET provider_id = '10000000-0000-0000-0000-000000000001' WHERE provider_id IS NULL;

-- Seed additional provider zones (e.g. for Hermes)
INSERT INTO shipping_zones (id, provider_id, zone_name, country_codes, is_default)
VALUES
('44444444-4444-4444-4444-444444444444', '10000000-0000-0000-0000-000000000002', 'Hermes Deutschland Privat', '["DE"]', false),
('55555555-5555-5555-5555-555555555555', '10000000-0000-0000-0000-000000000002', 'Hermes EU Borderless', '["AT", "FR", "NL", "BE", "PL"]', false)
ON CONFLICT DO NOTHING;

INSERT INTO shipping_rates (id, zone_id, name, package_type, min_weight_g, max_weight_g, price_cents, estimated_delivery_days)
VALUES
('ffffffff-1111-1111-1111-111111111111', '44444444-4444-4444-4444-444444444444', 'Hermes Päckchen S (Shop2Shop)', 'standard', 0, 3000, 370, '2-3 business days'),
('ffffffff-2222-2222-2222-222222222222', '44444444-4444-4444-4444-444444444444', 'Hermes Paket M (Haustürzustellung)', 'standard', 0, 10000, 540, '2-3 business days'),
('ffffffff-3333-3333-3333-333333333333', '55555555-5555-5555-5555-555555555555', 'Hermes International Parcel', 'standard', 0, 5000, 890, '3-5 business days')
ON CONFLICT DO NOTHING;

-- 2. Product Parts / Bill of Materials (BOM)
CREATE TABLE IF NOT EXISTS product_parts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES product_variants(id) ON DELETE CASCADE, -- NULL if universal to all variants
    part_name VARCHAR(255) NOT NULL,
    part_sku VARCHAR(100),
    quantity INTEGER NOT NULL DEFAULT 1,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_product_parts_product_id ON product_parts(product_id);
CREATE INDEX IF NOT EXISTS idx_product_parts_variant_id ON product_parts(variant_id);

-- Seed initial parts for Mechanical Keyboard
-- Universal parts:
INSERT INTO product_parts (product_id, variant_id, part_name, part_sku, quantity, notes)
VALUES
('a1000000-0000-0000-0000-000000000001', NULL, 'CNC Anodized 6063 Aluminum Chassis & Brass Weight', 'PRT-CASE-ALUM', 1, 'Precision CNC milled case with integrated sound dampening silicone'),
('a1000000-0000-0000-0000-000000000001', NULL, 'Hot-Swap PCB with RGB & Rotary Encoder (QMK/VIA)', 'PRT-PCB-75-RGB', 1, 'South-facing LED per-key lighting, 1000Hz polling rate'),
('a1000000-0000-0000-0000-000000000001', NULL, 'Screw-in PCB Stabilizers (Lubed with Krytox 205g0)', 'PRT-STAB-V2', 1, 'Factory tuned and wire-balanced'),
('a1000000-0000-0000-0000-000000000001', NULL, 'Braided Coiled USB-C to USB-A Aviator Cable', 'PRT-CBL-AVIATOR', 1, '1.5m custom paracord with metal 5-pin GX16 connector');

-- Variant-specific parts:
INSERT INTO product_parts (product_id, variant_id, part_name, part_sku, quantity, notes)
VALUES
-- Tactile Brown variant (b1000000-0000-0000-0000-000000000001)
('a1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000001', 'Gateron G Pro 3.0 Tactile Brown Switches (x84)', 'PRT-SW-BROWN-84', 1, '55g tactile bump, pre-lubed stem and leaf'),
('a1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000001', 'Double-Shot PBT Stealth Charcoal Keycaps Set', 'PRT-KC-PBT-BLK', 1, 'Cherry profile, oil-resistant 1.5mm thick walls'),

-- Linear Red variant (b1000000-0000-0000-0000-000000000002)
('a1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000002', 'Gateron G Pro 3.0 Linear Red Switches (x84)', 'PRT-SW-RED-84', 1, '45g smooth linear travel, dual-stage spring'),
('a1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000002', 'Double-Shot PBT Stealth Charcoal Keycaps Set', 'PRT-KC-PBT-BLK', 1, 'Cherry profile, oil-resistant 1.5mm thick walls'),

-- Ferris Clicky Blue variant (b1000000-0000-0000-0000-000000000003)
('a1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000003', 'Kailh Box Clicky Blue Switches (x84)', 'PRT-SW-BLUE-84', 1, 'Crisp clickbar tactile feedback, IP56 water resistant'),
('a1000000-0000-0000-0000-000000000001', 'b1000000-0000-0000-0000-000000000003', 'Ferris Signature Edition Orange/Dark Keycaps Set', 'PRT-KC-PBT-FERRIS', 1, 'Custom dyed Rust novelty keys included');

-- 3. CMS & Policy Pages Table
CREATE TABLE IF NOT EXISTS pages (
    slug VARCHAR(100) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content_markdown TEXT NOT NULL DEFAULT '',
    is_published BOOLEAN NOT NULL DEFAULT true,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed Default Policy Pages with Markdown
INSERT INTO pages (slug, title, content_markdown, is_published)
VALUES 
('shipment-policy', 'Shipment Policy', 
'# Shipping & Delivery Policy

Thank you for choosing **RustCraft Gear & Software**. We are dedicated to delivering your orders swiftly, securely, and transparently.

### 1. Order Processing Time
- All physical hardware orders placed before **14:00 CET** on business days are dispatched on the same day.
- Orders received on weekends or German statutory holidays will be processed on the next business day.
- You will receive a tracking link via email the moment your parcel leaves our central warehouse in Berlin.

### 2. Digital Product Delivery
- Digital eBooks, architecture guides, and software downloads are delivered **instantly**.
- Download links are presented on your order confirmation page immediately after checkout and are also sent to your registered email address.

### 3. Shipping Carriers & Packaging
We partner with top-tier carriers including **DHL Express & Paket**, **Hermes**, and **UPS Worldwide**. All items are securely packed in eco-friendly shock-absorbent packaging.

### 4. Overview of Current Rates & Delivery Times
Below is the live schedule of our supported shipping carriers, destinations, and pricing categories calculated directly by our logistics engine:', true),

('contact', 'Contact Information',
'# Contact Information

Have questions about our hardware, need order support, or want to discuss enterprise licensing? We are here to help.

### Customer Support & Operations
- **Email:** support@rustwebshop.local
- **Help Desk Hours:** Monday – Friday, 09:00 – 18:00 CET
- **Telephone:** +49 (0) 30 123456-78

### Headquarters & Logistics Center
**RustCraft Gear & Software GmbH**  
Rustacean Way 42  
10115 Berlin, Germany  

### Technical Inquiries
For bug reports, open-source crate questions, or API integration assistance, contact our core dev team at `dev@rustwebshop.local`.', true),

('legal-notice', 'Legal Notice (Impressum)',
'# Legal Notice / Impressum

Information pursuant to § 5 TMG (German Telemedia Act):

**Company Name:**  
RustCraft Gear & Software GmbH  
Rustacean Way 42  
10115 Berlin, Germany  

**Represented by Managing Directors:**  
Max Mustermann  

**Contact:**  
Phone: +49 (0) 30 123456-78  
Email: legal@rustwebshop.local  

**Commercial Register:**  
Amtsgericht Charlottenburg (Berlin)  
HRB 198421 B  

**VAT Identification Number (USt-IdNr.):**  
DE314159265  

**Online Dispute Resolution (ODR):**  
The European Commission provides a platform for online dispute resolution: [https://ec.europa.eu/consumers/odr](https://ec.europa.eu/consumers/odr).  
We are neither willing nor obligated to participate in dispute resolution proceedings before a consumer arbitration board.', true),

('terms-conditions', 'Terms and Conditions (AGB)',
'# General Terms and Conditions (AGB)

### § 1 Scope of Application
These General Terms and Conditions apply to all contracts concluded between **RustCraft Gear & Software GmbH** and consumers or business customers via this online store.

### § 2 Contract Conclusion
- Product presentations in our online shop constitute a binding offer to enter into a sales contract.
- By clicking the final "Pay & Complete Order" button, you accept the offer for the items in your shopping cart.
- Immediate confirmation of receipt and order details will be provided via email and on the order confirmation screen.

### § 3 Retention of Title
The delivered goods remain our property until full payment has been received.

### § 4 Warranty and Liability
Statutory warranty rights apply. For consumers in the European Union, the statutory warranty period is two years from delivery of the goods.

### § 5 Digital Goods
For software templates, license keys, and eBooks, access is granted immediately upon successful authorization of payment.', true),

('privacy-policy', 'Privacy Policy (Datenschutzerklärung)',
'# Privacy Policy (GDPR / DSGVO)

We take the protection of your personal data very seriously. We handle your personal data confidentially and in accordance with statutory data protection regulations (EU General Data Protection Regulation - GDPR).

### 1. Data Controller
RustCraft Gear & Software GmbH  
Rustacean Way 42, 10115 Berlin, Germany  
Email: privacy@rustwebshop.local  

### 2. Data Collection During Checkout
When placing an order, we collect:
- Full Name and Email Address
- Shipping and Billing Addresses
- Payment transaction references (processed directly by certified PCI-DSS Level 1 compliant processors: Stripe, PayPal, Apple Pay, Google Pay, Amazon Pay). We do not store raw credit card numbers on our servers.

### 3. Purpose of Processing
Your data is processed strictly for contract execution (Art. 6 Para. 1 lit. b GDPR), parcel delivery, and compliance with statutory fiscal retention obligations (Art. 6 Para. 1 lit. c GDPR).

### 4. Your Rights
You have the right to request information, rectification, deletion, or restriction of processing of your stored data at any time by contacting privacy@rustwebshop.local.', true),

('cookie-policy', 'Cookie Policy',
'# Cookie Policy

### Use of Essential Cookies
Our website strictly uses **functional session cookies** and local storage required for technical operation:
- **Shopping Cart Session:** To remember items placed in your cart while navigating between pages.
- **CSRF & Security Tokens:** To protect transaction integrity and prevent unauthorized cross-site requests.
- **Customer Authentication Token:** To maintain your authenticated state if you log in to your customer account.

We **do not** use third-party tracking or advertising cookies. No personal browsing habits are sold or profiled.', true),

('return-policy', 'Return & Cancellation Policy (Widerrufsrecht)',
'# Cancellation & Return Policy

### Statutory Right of Withdrawal (14 Days)
You have the right to withdraw from this contract within **14 days** without giving any reason. The withdrawal period will expire after 14 days from the day on which you acquire physical possession of the goods.

### Exercising Your Right of Withdrawal
To exercise your right of withdrawal, notify us via an unequivocal statement:
- **Email:** returns@rustwebshop.local
- **Postal:** RustCraft Gear & Software GmbH, Returns Department, Rustacean Way 42, 10115 Berlin, Germany

### Return Shipping
You will bear the direct cost of returning physical goods unless the item received was defective or incorrect. Returned goods must be undamaged and in original packaging.

### Exceptions to Withdrawal
The right of withdrawal does not apply to:
- Digital goods (software licenses, eBooks) once the download or stream has begun with your prior express consent.', true)
ON CONFLICT (slug) DO NOTHING;

-- 4. Dynamic Navigation Menu Items
CREATE TABLE IF NOT EXISTS navigation_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    label VARCHAR(100) NOT NULL,
    url VARCHAR(255) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM navigation_items) THEN
        INSERT INTO navigation_items (id, label, url, sort_order, is_active)
        VALUES
        ('20000000-0000-0000-0000-000000000001', 'Catalog', '/', 1, true),
        ('20000000-0000-0000-0000-000000000002', 'Hardware', '/?category=Hardware', 2, true),
        ('20000000-0000-0000-0000-000000000003', 'Apparel', '/?category=Apparel', 3, true),
        ('20000000-0000-0000-0000-000000000004', 'Digital & Books', '/?category=Software+%26+Books', 4, true),
        ('20000000-0000-0000-0000-000000000005', 'Shipping Policy', '/policies/shipment-policy', 5, true),
        ('20000000-0000-0000-0000-000000000006', 'Track Order', '/track', 6, true)
        ON CONFLICT DO NOTHING;
    END IF;
END $$;

-- 5. Customer Addresses and Wishlist
CREATE TABLE IF NOT EXISTS customer_addresses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_email VARCHAR(255) NOT NULL,
    address_type VARCHAR(20) NOT NULL DEFAULT 'shipping',
    full_name VARCHAR(255) NOT NULL,
    street_address VARCHAR(255) NOT NULL,
    apartment_suite VARCHAR(100),
    city VARCHAR(100) NOT NULL,
    state_province VARCHAR(100) NOT NULL,
    postal_code VARCHAR(50) NOT NULL,
    country_code VARCHAR(10) NOT NULL DEFAULT 'DE',
    is_default BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_customer_addresses_email ON customer_addresses(customer_email);

CREATE TABLE IF NOT EXISTS customer_wishlist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_email VARCHAR(255) NOT NULL,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_customer_product_wishlist UNIQUE (customer_email, product_id)
);

CREATE INDEX IF NOT EXISTS idx_customer_wishlist_email ON customer_wishlist(customer_email);
