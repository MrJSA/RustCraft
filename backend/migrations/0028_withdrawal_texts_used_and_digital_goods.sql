-- Migration 0028: clearer default withdrawal and return texts for digital files and goods that were used or processed
-- (e.g. soldered electronics). The statutory right of withdrawal cannot be excluded for used goods; the consumer owes
-- compensation for the loss in value instead (§ 357a BGB). Only the exact default sentences are replaced, so texts
-- the shop wrote itself stay as they are.

UPDATE pages
SET content_markdown = REPLACE(content_markdown,
        'You must pay for any depreciation of the products only if this depreciation can be attributed to handling other than what was necessary for checking the condition, features, and functionality of the products.',
        'You must pay for any loss in value of the products only if this loss in value is due to handling that was not necessary to check the condition, features and functionality of the products.

You may check the products as you could in a shop. Installing, assembling, soldering or otherwise processing them, or using them beyond such a check, is not necessary for checking them. If products can no longer be sold as new because of this, you must compensate us for the loss in value, which can amount to the full purchase price.'),
    updated_at = NOW()
WHERE slug = 'revocation-policy'
  AND content_markdown LIKE '%You must pay for any depreciation of the products only if this depreciation can be attributed to handling other than what was necessary for checking the condition, features, and functionality of the products.%';

UPDATE pages
SET content_markdown = REPLACE(content_markdown,
        '- For digital goods or downloadable software once the performance has begun with prior express consent.',
        '- The right of withdrawal expires for contracts for the supply of digital content that is not on a physical medium (e.g. downloadable files) once we have begun performance of the contract, after you have expressly agreed that we begin before the end of the withdrawal period and confirmed that you are aware that you thereby lose your right of withdrawal.'),
    updated_at = NOW()
WHERE slug = 'revocation-policy'
  AND content_markdown LIKE '%- For digital goods or downloadable software once the performance has begun with prior express consent.%';

UPDATE pages
SET content_markdown = REPLACE(content_markdown,
        'You can return products to us within **30 days** from the day of purchase, provided the product is still in brand new, original condition.',
        'In addition to the statutory right of withdrawal, you can return products to us within **30 days** from the day of purchase, provided the product is still in brand new, original condition.

### Excluded from Voluntary Returns
- **Digital products** (files, downloads), once the download has been made available to you.
- **Products that have been used or processed**, e.g. installed, assembled or soldered.

These conditions do not limit your statutory rights. Business customers have no statutory right of withdrawal.'),
    updated_at = NOW()
WHERE slug = 'return-policy'
  AND content_markdown LIKE '%You can return products to us within **30 days** from the day of purchase, provided the product is still in brand new, original condition.%';
