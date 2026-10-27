-- Migration 0008: Legal identity fields, German/EU compliance disclosures and policies
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS legal_name VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS store_owner VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS commercial_register VARCHAR(255) NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS dispute_resolution_notice TEXT NOT NULL DEFAULT '';
ALTER TABLE store_settings ADD COLUMN IF NOT EXISTS odr_url VARCHAR(500) NOT NULL DEFAULT 'https://ec.europa.eu/odr';

-- Populate default legal store information from store settings
UPDATE store_settings
SET 
    store_name = COALESCE(NULLIF(store_name, 'RustCraft Gear & Software GmbH'), 'RustCraft Store'),
    legal_name = 'Max Mustermann E-Commerce',
    store_owner = 'Max Mustermann',
    company_address = 'Musterstraße 1, 12345 Musterstadt, Germany',
    phone = '+49 123 4567890',
    support_email = 'shop@example.com',
    vat_id = 'DE123456789',
    tax_notice = 'Value added tax is not collected, as small businesses according to §19 (1) UStG.',
    odr_url = 'https://ec.europa.eu/odr',
    dispute_resolution_notice = 'The European Commission provides a platform for the out-of-court resolution of disputes (ODR platform), which can be viewed under https://ec.europa.eu/odr. We are not willing and not obligated to enter into dispute resolution proceedings before the consumer arbitration board.'
WHERE id = 1;

-- Ensure all legal pages exist and incorporate store placeholders
INSERT INTO pages (slug, title, content_markdown, is_published)
VALUES 
('revocation-policy', 'Revocation Policy & Model Form', 
'# Revocation Right for Consumers

(A consumer is any natural person who concludes a legal transaction which, to an overwhelming extent, cannot be attributed to either his commercial or independent professional activities.)

### Instructions for Revocation

#### Right of Revocation
You have the right to revoke this contract within **14 days** without specifying any reasons. The revocation period is 14 days with effect from the day on which you or a third party nominated by you, which is not the carrier, had taken possession of the products.

To exercise your right of withdrawal, you must inform us:
- **{{LEGAL_NAME}}** (Trade name: {{STORE_NAME}})
- **Address:** {{COMPANY_ADDRESS}}
- **Phone:** {{PHONE}}
- **E-Mail:** {{SUPPORT_EMAIL}}

by means of a clear declaration (e.g. a letter sent by post, or an e-mail) of your decision to withdraw from this contract. You can use the attached model withdrawal form for this purpose, which is, however, not mandatory.

In order to safeguard the revocation period, it is sufficient that you send the notification about the exercise of the revocation right before the expiry of the revocation period.

#### Consequences of the Revocation
If you revoke this contract, we shall repay all payments received from you, including delivery costs (with the exception of supplementary costs arising from your choice of a delivery method other than the least expensive standard delivery offered by us), without undue delay and at the latest within 14 days from the day on which we received the notification about the revocation. We use the same means of payment as you used for the initial transaction unless expressly agreed otherwise; no fees will be charged for this repayment.

We can refuse repayment until the products are returned to us or until you have furnished evidence that you have sent the products back to us, whichever is earlier.

You must return or transfer the products to us immediately and at the latest within 14 days with effect from the day on which you inform us of the revocation of this contract. The deadline is maintained if you send the products before the expiry of the 14-day deadline.

You bear the direct costs for returning the products.

You must pay for any depreciation of the products only if this depreciation can be attributed to handling other than what was necessary for checking the condition, features, and functionality of the products.

#### Criteria for Exclusion or Expiring
The revocation right is not available for contracts:
- For the delivery of products which are not prefabricated and for whose manufacturing an individual selection or stipulation by the consumer is decisive or which are clearly tailored to the personal requirements of the consumer.
- For the delivery of sealed products which are not suitable for return for reasons of health protection or hygiene if their seal has been removed after delivery.
- For the delivery of sound or video recordings or computer software in a sealed package if the seal has been removed after delivery.
- For digital goods or downloadable software once the performance has begun with prior express consent.

---

### Model Revocation Form (Muster-Widerrufsformular)

*(If you wish to revoke the contract, please fill out this form and send it back to us.)*

- **To:** {{LEGAL_NAME}}, {{COMPANY_ADDRESS}}, E-Mail: {{SUPPORT_EMAIL}}
- I/We (*) herewith revoke the contract concluded by me/us (*) regarding the purchase of the following products (*): 
- Ordered on (*) / Received on (*):
- Name of the consumer(s):
- Address of the consumer(s):
- Signature of the consumer(s) *(only in case of notification on paper)*:
- Date:
', true)
ON CONFLICT (slug) DO NOTHING;

-- Update legal-notice to standard § 5 TMG format with placeholders
UPDATE pages
SET title = 'Legal Notice (Impressum)',
    content_markdown = '# Legal Notice (Impressum)

According to § 5 TMG / § 55 RStV:

**{{LEGAL_NAME}}**  
Trade name: **{{STORE_NAME}}**  
{{COMPANY_ADDRESS}}  

### Represented by:
{{STORE_OWNER}}  

### Contact:
- **Phone:** {{PHONE}}  
- **Email:** {{SUPPORT_EMAIL}}  
- **VAT Identification Number (USt-IdNr.):** {{VAT_ID}}  

### Tax Status:
{{TAX_NOTICE}}  

### Online Dispute Resolution:
The European Commission provides a platform for out-of-court resolution of disputes (ODR platform): [{{ODR_URL}}]({{ODR_URL}}).  
{{DISPUTE_RESOLUTION_NOTICE}}  

---

### Liability for Content
The contents of our website have been created with the greatest possible care. However, we cannot guarantee the contents’ accuracy, completeness, or topicality. According to Section 7, paragraph 1 of the TMG (German Telemedia Act), we as service providers are liable for our content on these pages by general laws. However, according to Sections 8 to 10 of the TMG, service providers are not obliged to monitor external information transmitted or stored or investigate circumstances pointing to illegal activity.

### Liability for Links
Our website contains links to external websites, over whose contents we have no control. Therefore, we cannot accept any liability for these external contents. The respective provider or operator of the websites is always responsible for the contents of the linked pages.

### Copyright
The contents and works on these pages created by the site operator are subject to German copyright law. Duplication, processing, distribution, and any kind of utilization outside the limits of copyright require written consent of the respective author or creator.
'
WHERE slug = 'legal-notice';

-- Update contact page with placeholders
UPDATE pages
SET title = 'Contact Information',
    content_markdown = '# Contact Information

**{{LEGAL_NAME}}**  
Trade name: **{{STORE_NAME}}**  

### Postal Address:
{{COMPANY_ADDRESS}}  

### Communication Channels:
- **Phone:** {{PHONE}}  
- **Email:** {{SUPPORT_EMAIL}}  
- **VAT Identification Number:** {{VAT_ID}}  

### Help & Inquiries:
For customer support, order assistance, or custom modifications, reach out to us at [{{SUPPORT_EMAIL}}](mailto:{{SUPPORT_EMAIL}}).
'
WHERE slug = 'contact';

-- Update return-policy to reference the revocation rights
UPDATE pages
SET title = 'Return & Refund Policy',
    content_markdown = '# Return Policy

You can return products to us within **30 days** from the day of purchase, provided the product is still in brand new, original condition.

### Statutory Right of Revocation
Consumers have a statutory 14-day right of revocation. Please review our full [Revocation Policy & Model Form](/policies/revocation-policy) for complete details.

### Shipping Returns
As a specialized business, return postage costs are borne by the customer:
- **Return Address:** {{LEGAL_NAME}}, {{COMPANY_ADDRESS}}
- **Contact:** {{SUPPORT_EMAIL}}

Please ensure items are packed securely in their original packaging to prevent transit damage.
'
WHERE slug = 'return-policy';
